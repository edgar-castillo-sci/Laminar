"""Tests for the laminar Python bindings."""

import laminar
import pytest


def test_version():
    assert laminar.__version__
    assert laminar.version() == laminar.__version__


def test_modules():
    mods = laminar.modules()
    assert "honeycomb" in mods["symmetry"]
    assert "jam" in mods["notation"]
    assert "cif" in mods["export_formats"]


def test_sandwich_count():
    assert laminar.sandwich(n=6, option="PL", count=True) == 70
    assert laminar.sandwich(n=2, option="OneH", count=True) == 5
    assert laminar.sandwich(n=4, option="OneH", count=True) == 120


def test_sandwich_stdout():
    chains = laminar.sandwich(n=2, option="PL", export="stdout")
    assert len(chains) == 2
    assert all(isinstance(c, str) for c in chains)


def test_sandwich_unsupported_export():
    with pytest.raises(ValueError, match="unsupported export"):
        laminar.sandwich(n=2, option="PL", export="cif")


def test_sandwich_unknown_option():
    with pytest.raises(ValueError, match="unknown option"):
        laminar.sandwich(n=2, option="XYZ", count=True)


def test_compare_equivalent():
    result = laminar.compare(["+0+j+j", "+j+0+j"], option="PL")
    assert result["equivalent"] is True
    assert len(result["orbits"]) == 1


def test_compare_non_equivalent():
    result = laminar.compare(
        ["+0+j+j/+j+0+j", "+0+j+j/+0+j+j"], option="PL"
    )
    assert result["equivalent"] is False
    assert len(result["orbits"]) == 2


def test_compare_invalid_chain():
    with pytest.raises(ValueError, match="chain"):
        laminar.compare(["invalid"], option="PL")


def test_jamchain_parse():
    chain = laminar.JamChain.parse("+0+j+j", option="PL")
    assert chain.raw == "+0+j+j"
    assert chain.canonical == "+0+j+j"
    assert chain.k == 1
    assert chain.option == "PL"


def test_jamchain_canonical():
    chain = laminar.JamChain.parse("+j+0+j", option="PL")
    assert chain.canonical == "+0+j+j"


def test_jamchain_oneh():
    # Single layer: palindromic (trivially), not periodic (m >= 2 needed)
    # => cell = pP, not PP.
    chain = laminar.JamChain.parse("*j+0(+k-k)", option="OneH")
    assert chain.k == 1
    assert chain.cell == "pP"
    assert chain.is_palindromic is True
    assert chain.is_periodic is False


def test_jamchain_oneh_2layer_is_pp():
    # Two identical layers: palindromic + periodic (period 1) => PP.
    chain = laminar.JamChain.parse("*j+0(+k-k)/*j+0(+k-k)", option="OneH")
    assert chain.k == 2
    assert chain.cell == "PP"
    assert chain.is_palindromic is True
    assert chain.is_periodic is True
    assert chain.period == 1


def test_jamchain_invalid():
    with pytest.raises(ValueError, match="row"):
        laminar.JamChain.parse("invalid", option="PL")


def test_reference_cell_constructor():
    cell = laminar.ReferenceCell(
        a=3.16, c=12.30, z_up=1.56, z_down=-1.56,
        species_map={"j": "Mo", "k": "S"},
    )
    assert cell.a == 3.16
    assert cell.species_map["j"] == "Mo"


def test_reference_cell_from_toml(tmp_path):
    p = tmp_path / "cell.toml"
    p.write_text(
        'a = 3.16\nc = 12.30\nz_up = 1.56\nz_down = -1.56\n'
        '[species_map]\nj = "Mo"\nk = "S"\n'
    )
    cell = laminar.ReferenceCell.from_toml(str(p))
    assert cell.a == 3.16
    assert cell.species_map["j"] == "Mo"


def test_jamchain_to_cif():
    cell = laminar.ReferenceCell(
        a=3.16, c=12.30, z_up=1.56, z_down=-1.56,
        species_map={"j": "Mo", "k": "S"},
    )
    chain = laminar.JamChain.parse("*j+0(+k-k)", option="OneH")
    cif = chain.to_cif(cell)
    assert "_cell_length_a" in cif
    assert "Mo" in cif
    assert "S" in cif
