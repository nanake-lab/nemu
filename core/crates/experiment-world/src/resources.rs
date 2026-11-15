//! Fixed synthetic resources copied as values, never copied algorithms.
use core_kernel::{dynamics::*, fast::DriveLaw, law};
use core_types::{contract::Limits, generated::*};
pub const LIMITS: Limits = Limits {
    max_bytes: 1 << 20,
    max_items: 4096,
    max_depth: 32,
};
pub const VERSION: &str = "P5-30-L0-control-2";
pub fn vector(a: [f64; 4]) -> NatureVector {
    NatureVector {
        schema_version: 1,
        connection: a[0],
        empathy: a[1],
        curiosity: a[2],
        achievement: a[3],
    }
}
pub fn scale(a: [f64; 4]) -> NatureScale {
    NatureScale {
        schema_version: 1,
        connection: a[0],
        empathy: a[1],
        curiosity: a[2],
        achievement: a[3],
    }
}
pub fn concept(id: &str) -> ConceptRef {
    ConceptRef {
        schema_version: 1,
        concept_id: id.into(),
        registry_version: VERSION.into(),
    }
}
pub fn text(s: &str) -> TrilingualText {
    TrilingualText {
        schema_version: 1,
        zh: s.into(),
        en: s.into(),
        ja: s.into(),
    }
}
// All values: P5-26 tests/fixture.rs dynamics("old-v1"). Uncalibrated.
pub fn dynamics() -> DynamicsLaw {
    DynamicsLaw {
        version: VERSION.into(),
        baseline_prior: [0.; 4],
        gain_prior: [1.; 4],
        sigma_prior: [0.3; 4],
        tau_prior_s: [10.; 4],
        baseline_weight: 100.,
        variance_weight: 10.,
        duration_weight: 10.,
        change_prior: 0.05,
        change_prior_weight: 10.,
        change_min: 0.01,
        gain_min: 0.2,
        gain_max: 3.,
        recovery_min: 0.001,
        recovery_max: 2.,
        detector_shift_sigma: 1.,
        detector_threshold: 8.,
    }
}
// P5-26 tests/fixture.rs parameters(1.0), as used by Legacy::new old-v1.
pub fn parameters() -> LawParameters {
    LawParameters {
        schema_version: 1,
        n0: 1.,
        kappa1: 0.1,
        w_max: 8.,
        w_teach: 1.,
        v_u: 0.25,
        r_min: 0.1,
        r_max: 1.,
        h0: 0.01,
        n_h: 10.,
        h_hat_min: None,
        h0_theta: 0.1,
        h_hat_theta_min: 0.01,
        n_b: 10.,
        n_sigma: 10.,
        g_min: 0.2,
        g_max: 3.,
        recovery_min_per_s: 0.01,
        recovery_max_per_s: 10.,
        w_min: 0.1,
        t_x_s: 60.,
        epsilon_x: 0.01,
        k_max: 0.5,
    }
}
// P5-26 tests/legacy.rs; notices are recorded, never used to stop growth.
pub fn fuses() -> DynamicsFuses {
    DynamicsFuses {
        jump_notice: 100.,
        drift_run: 1_000_000,
        drift_margin: 0.01,
    }
}
pub fn drive_law() -> DriveLaw {
    DriveLaw {
        g_max: dynamics().gain_max,
        r_max: parameters().r_max,
        w_min: parameters().w_min,
        kappa: 0.5,
    }
}
pub fn relations() -> law::RelationConcepts {
    law::RelationConcepts {
        closeness: concept("closeness"),
        trust: concept("trust"),
    }
}
pub fn reference() -> ObjectRef {
    use core_types::scalar::{DecimalU64, HexBytes};
    use sha2::{Digest, Sha256};
    let bytes = binding_bytes();
    ObjectRef {
        schema_version: 1,
        digest: HexBytes(Sha256::digest(&bytes).to_vec()),
        object_type: "echo.law-version".into(),
        resource_version: VERSION.into(),
        byte_length: DecimalU64(bytes.len() as u64),
    }
}
pub fn binding_bytes() -> Vec<u8> {
    serde_json::to_vec(&serde_json::json!({"version":VERSION,"g":parameters(),"c_explicit_fields":format!("{:?}",dynamics()),"f":format!("{:?}",drive_law()),"fuses":format!("{:?}",fuses()),"capacity":100,"protected":1,"attachment_prior":0,"genesis_r":[],"genesis_x":[0,0,0,0]})).unwrap()
}
