use core_types::{
    contract::Limits,
    generated::{InitialPublication, InitialRuntimeRoot, LineageHeaderV2},
    scalar::{hex, unhex},
    semantic::{check_initial_store_binding, verify_birth_material},
};
use serde_json::Value;

const LIMITS: Limits = Limits {
    max_bytes: 1 << 20,
    max_items: 4096,
    max_depth: 32,
};

fn fixtures() -> Vec<Value> {
    serde_json::from_str(include_str!("../../../contracts/golden/legal.json")).unwrap()
}

#[test]
fn initial_root_binds_the_receiving_store_and_actual_stage_set() {
    let all = fixtures();
    let root: InitialPublication = serde_json::from_value(
        all.iter()
            .map(|v| &v["request"])
            .find(|q| q["name"] == "InitialPublication" && q["value"]["header"]["lineage"] == 3)
            .unwrap()["value"]
            .clone(),
    )
    .unwrap();
    let check = |store: &str, branch: &str, header: &LineageHeaderV2, q: &InitialRuntimeRoot| {
        check_initial_store_binding(&root, store, branch, header, q, LIMITS)
    };
    let mut connected = root.q.clone();
    connected.environment.implementations.reverse(); // Ω.implementations is a set.
    check(&root.store_id, &root.branch_id, &root.header, &connected).unwrap();
    assert!(check("other store", &root.branch_id, &root.header, &connected).is_err());
    assert!(check(&root.store_id, "other branch", &root.header, &connected).is_err());
    let mut other_world = root.header.clone();
    other_world.world_id = Some("different synthetic world".into());
    assert!(check(&root.store_id, &root.branch_id, &other_world, &connected).is_err());
    connected.fast = None;
    assert!(check(&root.store_id, &root.branch_id, &root.header, &connected).is_err());
}

#[test]
fn birth_uses_the_signed_manifest_version_and_authenticated_session() {
    let all = fixtures();
    for (seed, tag) in [(1, "echo.genesis-v3"), (4, "echo.genesis-v2")] {
        let row = all
            .iter()
            .map(|v| &v["request"])
            .find(|q| {
                q["name"] == "InitialPublication"
                    && q["value"]["birth"]["manifest"]["object_type"] == tag
            })
            .unwrap();
        let root: InitialPublication = serde_json::from_value(row["value"].clone()).unwrap();
        let birth = root.birth.unwrap();
        let expected = all
            .iter()
            .map(|v| &v["expected"])
            .find(|v| v["digest"] == hex(&birth.manifest.digest.0))
            .unwrap();
        let manifest = unhex(expected["canonical"].as_str().unwrap()).unwrap();
        let signature = format!("synthetic signature fixture {seed}").into_bytes();
        let user = &birth.caretaker;
        verify_birth_material(
            &birth,
            user,
            &manifest,
            &signature,
            LIMITS,
            |signer, doc, sig| {
                assert_eq!(signer, user);
                assert_eq!(doc, manifest);
                assert_eq!(sig, signature);
                Ok(())
            },
        )
        .unwrap();
        assert!(
            verify_birth_material(&birth, user, &manifest, &signature, LIMITS, |_, _, _| Err(
                "signature rejected by verifier".into()
            ))
            .is_err()
        );
        let mut stranger = user.clone();
        stranger.session_id.push('X');
        let mut wrong_person = birth.clone();
        wrong_person.caretaker.principal_id.push('X');
        let mut altered = signature.clone();
        altered[0] ^= 1;
        let mut wrong_digest = birth.clone();
        wrong_digest.manifest.digest.0[0] ^= 1;
        for (b, caller, doc, sig) in [
            (&birth, &stranger, manifest.as_slice(), signature.as_slice()),
            (
                &wrong_person,
                &wrong_person.caretaker,
                &manifest,
                &signature,
            ),
            (&wrong_digest, user, &manifest, &signature),
            (&birth, user, &manifest[1..], &signature),
            (&birth, user, &manifest, &altered),
        ] {
            assert!(
                verify_birth_material(b, caller, doc, sig, LIMITS, |_, _, _| panic!(
                    "unbound material reached signature verifier"
                ))
                .is_err()
            );
        }
    }
}
