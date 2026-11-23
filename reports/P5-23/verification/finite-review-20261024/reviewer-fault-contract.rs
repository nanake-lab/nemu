use super::*;
use super::tests::{fixture,world::{Fixture,theta}};
use std::collections::BTreeMap;

fn full_g(r_min:f64, mu:f64, appraisal:f64)->LawResult<Output>{
    let mut p=fixture::parameters(3.,1.,0.1,10.);p.r_min=r_min;
    let mut f=Fixture::new(10);f.law=Law::new(&p).unwrap();
    let candidate=Candidate{record_id:"review-k".into(),concept:fixture::concept("review-concept"),predicate:vec![fixture::text("review predicate")],direction:fixture::vector([1.,0.,0.,0.]),relation:None};
    let group=GroupItem{group_id:"review-g".into(),position:0,source_key:"review-source".into(),members:vec!["review-e".into()],verifiable:true,content:GroupContent::Experience{appraisals:vec![fixture::vector([appraisal,0.,0.,0.])],matches:vec![Match{target:MatchTarget::Tendency("review-k".into()),mu}]}};
    let r=f.r0();let before=r.clone();let aliases=BTreeMap::new();
    let result=f.run(&r,&[],&theta([0.;4],[0.2;4]),&Step{revision:"review",candidates:&[candidate],groups:&[group],aliases:&aliases});
    assert_eq!(r,before);
    if let Ok(out)=&result{
        let t=out.r.tendencies.iter().find(|t|t.tendency_id=="review-k").unwrap();
        let a=&out.groups[0].allocations[0];
        println!("FAULT_CONTRACT_OBSERVATION r_min={r_min:e} mu={mu:e} p={appraisal:e} S={} W={} share={} q={} A={} V={} pi={} N={}",a.support,a.intensity,a.share,a.actual_retention,t.evidence_a,t.evidence_v,t.pi,t.group_count.0);
    }
    result
}

#[test]
fn full_g_must_report_nonfinite_support_ratio(){
    let r_min=f64::from_bits(1);let p=0.75;
    assert!((2.*p/r_min).is_infinite());
    let result=full_g(r_min,1.,p);
    assert!(matches!(result,Err(LawError::NonFinite(_))),"nonfinite 2*p/r_min was clipped and accepted");
}

#[test]
fn full_g_positive_support_underflow_diagnostic(){
    let mu=f64::from_bits(1);let clipped=2.*0.75/1.-1.;
    assert!(clipped>0. && mu>0. && mu*clipped==0.);
    let result=full_g(1.,mu,0.75);
    println!("POSITIVE_SUPPORT_UNDERFLOW_RETURNED_OK={}",result.is_ok());
}
