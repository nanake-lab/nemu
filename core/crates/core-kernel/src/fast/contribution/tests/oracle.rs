use super::*;

fn decode(v: &serde_json::Value) -> f64 {
    f64::from_bits(u64::from_str_radix(v.as_str().unwrap(), 16).unwrap())
}

#[test]
fn e09_binary64_matches_independently_rounded_rational_vectors() {
    let data: serde_json::Value = serde_json::from_str(include_str!("oracle.json")).unwrap();
    for v in data["vectors"].as_array().unwrap() {
        let input = &v["inputs"];
        let (x, h) = coordinate(decode(&input[0]), decode(&input[1]), decode(&input[2])).unwrap();
        bits(x, decode(&v["expected"][0]));
        bits(h, decode(&v["expected"][1]));
        let (xx, hh) = coordinate(x, h, decode(&input[2])).unwrap();
        bits(x, xx);
        bits(h, hh);
        assert!(
            h >= decode(&input[1]).min(decode(&input[2]))
                && h <= decode(&input[1]).max(decode(&input[2]))
        );
    }
    assert_eq!(data["exact_distance_comparisons"], 3125);
    println!(
        "U1 exact rational maximum decomposition residual: {}",
        data["max_decomposition_residual"]
    );
}

#[test]
fn ac2_15_i05_equal_slots_same_drive_and_different_decomposition_counterexample() {
    // Dyadic grid has exact arithmetic, so no false general binary64 claim.
    for x in [-1.0, -0.5, 0.0, 0.5, 1.0] {
        for xx in [-1.0, -0.5, 0.0, 0.5, 1.0] {
            for h in [-1.0, 0.0, 1.0] {
                for v in [-1.0, -0.5, 0.0, 0.5, 1.0] {
                    let (y, j) = coordinate(x, h, v).unwrap();
                    let (yy, jj) = coordinate(xx, h, v).unwrap();
                    assert!((y - yy).abs() + (j - jj).abs() <= (x - xx).abs());
                }
            }
        }
    }
    let (a, _) = coordinate(1.0, 1.0, 0.0).unwrap();
    let (b, _) = coordinate(1.0, 0.0, 0.0).unwrap();
    bits(a, 0.0);
    bits(b, 1.0); // equal x does not mean equal extended state
}

#[test]
fn b6_fixed_target_and_b6_prime_updated_history_enclosures() {
    for target in [-1.0, -0.25, 0.0, 0.5, 1.0] {
        let mut h = 0.0;
        for x in [0.0, 1.0, -1.0, 0.5, -0.5].into_iter().cycle().take(100) {
            let (_, j) = coordinate(x, h, target).unwrap();
            h = j;
            assert!(h >= target.min(0.0) && h <= target.max(0.0));
            h = finite(h + 0.5 * (0.0 - h), "fixture decay").unwrap();
        }
    }
    // Updated targets enlarge the historical enclosure, never shrink it.
    let (mut h, mut lo, mut hi) = (0.0, 0.0_f64, 0.0_f64);
    for (x, v) in [(0.0, 1.0), (1.0, -1.0), (1.0, 0.0), (-1.0, 0.25)] {
        lo = lo.min(v);
        hi = hi.max(v);
        h = coordinate(x, h, v).unwrap().1;
        assert!(h >= lo && h <= hi);
    }
    // Exact Python rounding oracle for successive decimal merges.
    let data: serde_json::Value = serde_json::from_str(include_str!("oracle.json")).unwrap();
    let mut sum = 0.0;
    for (i, v) in [0.1, 0.2, 0.3].into_iter().enumerate() {
        sum = finite(sum + v, "merge fixture").unwrap();
        bits(sum, decode(&data["merge_expected"][i]));
    }
}
