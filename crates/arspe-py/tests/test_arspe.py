"""arspe_py 绑定测试：合成数据 + 真实样例（.session/fixtures，缺失时跳过）。"""

import json
import struct
from pathlib import Path

import numpy as np
import pytest

import arspe_py as arspe

H_PLANCK, C0, E_CHARGE = 6.62607015e-34, 299792458.0, 1.602176634e-19
CONST_1240 = H_PLANCK * C0 * 1e9 / E_CHARGE

FIXTURES = Path(__file__).resolve().parents[3] / ".session" / "fixtures"


# ---- 合成 .spe ----

def make_spe(width, height, counts=1, samples_i32=(), xml=None):
    data = bytearray(4100 + len(samples_i32) * 4)
    struct.pack_into("<H", data, 6, width)
    struct.pack_into("<H", data, 18, height)
    data[1446] = counts
    for i, v in enumerate(samples_i32):
        struct.pack_into("<i", data, 4100 + i * 4, v)
    if xml:
        data += xml.encode()
    return bytes(data)


def make_row_major_text(rows, cols):
    lines = ["\t".join(f"{400.0 + i:.1f}" for i in range(cols))]
    for r in range(1, rows + 1):
        lines.append("\t".join(f"{r * 1000.0 + i:.1f}" for i in range(cols)))
    return "\n".join(lines)


@pytest.fixture
def synthetic_spe_bytes():
    return make_spe(2, 3, samples_i32=[1, 2, 3, 4, 5, 6])


class TestParse:
    def test_parse_spe_bytes(self, synthetic_spe_bytes):
        spe = arspe.parse_spe(synthetic_spe_bytes)
        assert (spe.width, spe.height, spe.frame_count) == (2, 3, 1)
        assert spe.frames.dtype == np.float64
        assert spe.frames.shape == (1, 3, 2)
        assert np.array_equal(spe.frames[0], [[1, 2], [3, 4], [5, 6]])
        assert np.array_equal(spe.min_max, [[1.0, 6.0]])
        assert spe.wavelength is None

    def test_open_file_spe(self, tmp_path, synthetic_spe_bytes):
        p = tmp_path / "sample.spe"
        p.write_bytes(synthetic_spe_bytes)
        spe = arspe.open_file(p)
        assert np.array_equal(spe.frames[0], [[1, 2], [3, 4], [5, 6]])

    def test_open_file_txt(self, tmp_path):
        p = tmp_path / "sample.txt"
        p.write_text(make_row_major_text(3, 101), encoding="utf-8")
        spe = arspe.open_file(p)
        assert (spe.width, spe.height) == (101, 3)
        wl = spe.wavelength
        assert wl is not None and len(wl) == 101
        assert wl[0] == 400.0 and wl[-1] == 500.0
        assert spe.frames[0][0, 0] == 1000.0
        assert spe.frames[0][2, 100] == 3100.0

    def test_open_file_missing(self):
        with pytest.raises(arspe.ParseError):
            arspe.open_file("nonexistent_某某.spe")

    def test_open_file_bad_extension(self, tmp_path):
        # 与 GUI 行为一致：未知扩展名的文本文件走文本解析，
        # 解析不出数据时报"未能解析到有效数据"（ParseError）
        p = tmp_path / "data.unknown"
        p.write_text("hello")
        with pytest.raises(arspe.ParseError):
            arspe.open_file(p)

    def test_parse_text_invalid(self):
        with pytest.raises(arspe.ParseError):
            arspe.parse_text("hello world")

    def test_exception_hierarchy(self):
        assert issubclass(arspe.ParseError, arspe.ArspeError)
        assert issubclass(arspe.UnsupportedFormatError, arspe.ArspeError)
        assert issubclass(arspe.StretchError, arspe.ArspeError)


class TestSpeDataObject:
    def test_calibration(self, synthetic_spe_bytes):
        spe = arspe.parse_spe(synthetic_spe_bytes)
        assert spe.calibration.detector_angle_cal == 0.0

    def test_to_json_round_trip(self, synthetic_spe_bytes):
        spe = arspe.parse_spe(synthetic_spe_bytes)
        d = json.loads(spe.to_json())
        assert d["width"] == 2 and d["height"] == 3
        assert d["frame"] == [[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]]
        assert d == spe.to_dict()

    def test_save_csv(self, tmp_path):
        spe = arspe.parse_text(make_row_major_text(3, 101))
        p = tmp_path / "out.csv"
        spe.save_csv(p)
        lines = p.read_text().splitlines()
        assert lines[0].split("\t")[0] == "y\\x"
        # 行 = 波长，列 = 角度行索引；矩阵值 = frames[frame][x, y]
        first_data = lines[1].split("\t")
        assert float(first_data[0]) == 400.0
        assert float(first_data[1]) == spe.frames[0][0, 0]

    def test_save_csv_frame_out_of_range(self, tmp_path, synthetic_spe_bytes):
        spe = arspe.parse_spe(synthetic_spe_bytes)
        with pytest.raises(ValueError):
            spe.save_csv(tmp_path / "o.csv", frame_index=5)

    def test_repr(self, synthetic_spe_bytes):
        assert "width=2" in repr(arspe.parse_spe(synthetic_spe_bytes))


class TestStretch:
    def test_tan_identity_without_ev(self):
        spe = arspe.parse_text(make_row_major_text(3, 101))
        out = arspe.stretch(spe, False, "tan", 0.0, 1.0)
        assert np.array_equal(out.frames, spe.frames)
        assert np.array_equal(out.wavelength, spe.wavelength)

    def test_ev_mode_remaps_wavelength(self):
        spe = arspe.parse_text(make_row_major_text(3, 101))
        out = arspe.stretch(spe, True, "tan", 0.0, 2.0)
        wl = out.wavelength
        assert len(wl) == 101
        # eV 等间隔重映射后首尾波长不变
        assert abs(wl[0] - 400.0) < 1e-6
        assert abs(wl[-1] - 500.0) < 1e-6
        # 内部不再是均匀波长分布
        assert not np.allclose(np.diff(wl), np.diff(wl)[0])

    def test_ev_mode_requires_wavelength(self):
        spe = arspe.parse_spe(make_spe(2, 2, samples_i32=[1, 2, 3, 4]))
        with pytest.raises(arspe.StretchError):
            arspe.stretch(spe, True, "tan", -1.0, 1.0)
        with pytest.raises(arspe.StretchError):
            arspe.stretch(spe, False, "k", -1.0, 1.0)

    def test_stretch_input_unchanged(self):
        spe = arspe.parse_text(make_row_major_text(3, 101))
        before = spe.frames.copy()
        arspe.stretch(spe, True, "angle", 0.0, 0.5)
        assert np.array_equal(spe.frames, before)


# ---- 物理常数换算（P2 暴露的函数，此处与独立计算的 golden 值对拍）----

class TestConversions:
    def test_lambda_energy_round_trip(self):
        for nm in (400.0, 532.0, 850.0):
            assert arspe.lambda_to_energy(nm) == pytest.approx(CONST_1240 / nm, rel=1e-12)
            assert arspe.energy_to_lambda(CONST_1240 / nm) == pytest.approx(nm, rel=1e-12)

    def test_532nm_energy_value(self):
        # 文献值 ~2.33 eV
        assert arspe.lambda_to_energy(532.0) == pytest.approx(2.3309, abs=1e-3)

    def test_tan_angle_round_trip(self):
        for deg in (5.0, 15.0, 30.0):
            t = arspe.angle_to_tan(deg)
            assert arspe.tan_to_angle(t) == pytest.approx(deg, rel=1e-12)

    def test_tan_k_matches_formula(self):
        lam = 850.0
        k = arspe.tan_to_k(0.35, lam)
        assert k == pytest.approx(2 * np.pi * 0.35 / lam * 1000.0, rel=1e-12)
        assert arspe.k_to_tan(k, lam) == pytest.approx(0.35, rel=1e-12)

    def test_yrange(self):
        yr = arspe.YRange(400.0, 1000.0)
        assert yr.min_lambda == 400.0 and yr.max_lambda == 1000.0
        assert yr.min_energy == pytest.approx(CONST_1240 / 1000.0, rel=1e-12)
        assert yr.max_energy == pytest.approx(CONST_1240 / 400.0, rel=1e-12)
        assert yr.band_width == pytest.approx(600.0, rel=1e-12)
        lo, hi = yr.index_range_of_lambda_in(arspe.YRange(400.0, 1000.0), 100)
        assert (lo, hi) == (0, 100)
        # 子区间 [400, 1000] 落在全域 [400, 1400] 的前 60%
        lo, hi = yr.index_range_of_lambda_in(arspe.YRange(400.0, 1400.0), 100)
        assert (lo, hi) == (0, 60)

    def test_yrange_energy_index_mapping(self):
        yr = arspe.YRange(400.0, 1000.0)
        lo, hi = yr.index_range_of_energy_in(arspe.YRange(400.0, 1000.0), 100)
        assert (lo, hi) == (0, 100)

    def test_xrange_basic(self):
        xr = arspe.XRange(0.5, 850.0)
        tan_na = np.tan(np.arcsin(0.5))
        assert xr.max_tan == pytest.approx(tan_na, rel=1e-12)
        assert xr.min_tan == pytest.approx(-tan_na, rel=1e-12)
        assert xr.max_angle == pytest.approx(30.0, rel=1e-12)  # asin(0.5) = 30°

    def test_xrange_bound_and_k(self):
        # TS 绑定模式公式：minTan = -t - 2t/(to-from)·from，
        # (0, 50, 100) → minTan = -t，maxTan = -t + 4t = 3t
        xr = arspe.XRange(0.5, 850.0, binding=(0, 50, 100))
        tan_na = np.tan(np.arcsin(0.5))
        assert xr.min_tan == pytest.approx(-tan_na, rel=1e-12)
        assert xr.max_tan == pytest.approx(3 * tan_na, rel=1e-12)
        # 对全域 XRange 的索引映射：[(-t+t)/2t·100, (3t+t)/2t·100] = [0, 200]
        lo, hi = xr.index_range_of_tan_in(arspe.XRange(0.5, 850.0), 100)
        assert (lo, hi) == (0, 200)



# ---- 真实样例 ----

@pytest.fixture(scope="class")
def real_spe():
    return arspe.open_file(FIXTURES / "100.spe")


@pytest.fixture(scope="class")
def real_text():
    return arspe.open_file(FIXTURES / "power_3.asc")


@pytest.mark.skipif(not (FIXTURES / "100.spe").exists(), reason="缺少真实样例")
class TestRealSpe:
    def test_shapes_consistent(self, real_spe):
        spe = real_spe
        n, h, w = spe.frames.shape
        assert (h, w) == (spe.height, spe.width)
        assert spe.min_max.shape == (n, 2)
        lo, hi = spe.min_max[:, 0], spe.min_max[:, 1]
        assert (lo <= hi).all()
        assert (spe.frames.min(axis=(1, 2)) == lo).all()
        assert (spe.frames.max(axis=(1, 2)) == hi).all()

    def test_parse_bytes_matches_file(self, real_spe):
        spe = real_spe
        raw = (FIXTURES / "100.spe").read_bytes()
        assert np.array_equal(arspe.parse_spe(raw).frames, spe.frames)

    def test_stretch_smoke(self, real_spe):
        spe = real_spe
        out = arspe.stretch(spe, False, "angle", -0.3, 0.3)
        assert out.frames.shape == spe.frames.shape
        assert np.isfinite(out.frames).all()
        if spe.wavelength is not None:
            out_ev = arspe.stretch(spe, True, "k", -0.3, 0.3)
            assert np.isfinite(out_ev.frames).all()


@pytest.mark.skipif(not (FIXTURES / "power_3.asc").exists(), reason="缺少真实样例")
class TestRealText:
    def test_shapes_consistent(self, real_text):
        spe = real_text
        assert spe.wavelength is not None
        n, h, w = spe.frames.shape
        assert (n, h, w) == (1, spe.height, spe.width)
        assert w == len(spe.wavelength)
        assert np.isfinite(spe.frames).all()

    def test_stretch_smoke(self, real_text):
        spe = real_text
        out = arspe.stretch(spe, True, "tan", -0.2, 0.2)
        assert out.frames.shape == spe.frames.shape
        assert np.isfinite(out.frames).all()
