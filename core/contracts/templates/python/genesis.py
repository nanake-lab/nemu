"""Pure V2/V3 genesis rules, mirroring core-types semantic.rs (v2.0 §§3,8,13).

Codec validity alone is insufficient. No resource provenance or authority is inferred.
"""

from typing import Any

from .runtime import Limits, fail, normalize


def check_law_parameters(p: dict[str, Any]) -> None:
    """The existing Rust law inequalities; p has already been normalized."""
    fail(p["n0"] > 0, "n0 must be positive")
    fail(p["w_teach"] <= p["w_max"], "w_teach must not exceed w_max")
    fail(p["v_u"] < 1, "v_u must be below 1")
    fail(p["r_min"] <= p["r_max"], "r_min must not exceed r_max")
    fail(p["g_min"] <= p["g_max"], "g_min must not exceed g_max")
    fail(p["recovery_min_per_s"] > 0, "lambda_min must be positive")
    fail(
        p["recovery_min_per_s"] <= p["recovery_max_per_s"],
        "lambda_min must not exceed lambda_max",
    )


def check_genesis(name: str, value: dict[str, Any], limits: Limits) -> None:
    """Mandatory pure semantic check for either declared genesis version."""
    fail(name in ("GenesisManifestV2", "GenesisManifestV3"), "genesis type")
    g = normalize(name, value, limits)
    p = g["law_version"]["parameters"]
    check_law_parameters(p)
    gain = [g["initial_gain"][k] for k in ("connection", "empathy", "curiosity", "achievement")]
    fail(all(p["g_min"] <= x <= p["g_max"] for x in gain), "initial gain outside [g_min, g_max]")
    fail(all(gain[3] < x for x in gain[:3]), "achievement must have the weakest initial gain")
    fail(g["sigma0"] > 0, "sigma0 must be positive")
    fail(g["tau0_s"] > 0, "tau0 must be positive")
