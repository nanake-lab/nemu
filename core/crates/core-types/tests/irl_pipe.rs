use core_types::{
    WireMessage,
    contract::{Limits, Result},
    generated::*,
    irl_pipe::{Direction, PipeCall, frame_length},
};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::sync::OnceLock;

const LIMITS: Limits = Limits {
    max_bytes: 1048576,
    max_depth: 32,
    max_items: 4096,
};

fn fixture<T: WireMessage + DeserializeOwned>(select: impl Fn(&Value) -> bool) -> T {
    static GOLDEN: OnceLock<Vec<Value>> = OnceLock::new();
    let values = GOLDEN.get_or_init(|| {
        serde_json::from_slice(include_bytes!("../../../contracts/golden/legal.json")).unwrap()
    });
    let request = values
        .iter()
        .map(|v| &v["request"])
        .find(|q| q["name"] == T::NAME && select(&q["value"]))
        .unwrap();
    serde_json::from_value(request["value"].clone()).unwrap()
}

fn frame(op: IrlPipeOperation) -> IrlPipeFrame {
    fixture(|v| v["operation"] == u32::from(op))
}

fn apply(
    call: &mut PipeCall<'_>,
    direction: Direction,
    frame: &IrlPipeFrame,
) -> Result<IrlPipeFrame> {
    call.receive(direction, &frame.protobuf_bytes(LIMITS)?, LIMITS, |_| {
        Ok(&[])
    })
}

fn core_opening() -> (IrlPipeFrame, ProposalBindingV2) {
    let opening = frame(IrlPipeOperation::IRL_DISPATCH);
    let d = opening.dispatch.as_ref().unwrap();
    let mut b: ProposalBindingV2 = fixture(|_| true);
    b.branch_id.clone_from(&d.target.branch_id);
    b.event_id.clone_from(&d.target.record_id);
    b.c_core_digest.clone_from(&d.facts.object.digest);
    b.environment.registry.clone_from(&d.registry.object);
    b.read_set = vec![
        d.payload.object.clone(),
        d.facts.object.clone(),
        d.registry.object.clone(),
    ];
    (opening, b)
}

#[test]
fn borrow_requires_outstanding_request_visible_frontier_and_matching_bytes_and_charges_once() {
    use Direction::*;
    use IrlPipeOperation::*;
    let (opening, binding) = core_opening();
    let mut previous_binding = binding.clone();
    previous_binding.event_id.push_str("-previous");
    assert!(
        PipeCall::new(
            opening.clone(),
            IrlPipeLink::IRL_CORE_WORKER,
            Some(&previous_binding),
            &fixture(|_| true)
        )
        .is_err()
    );
    let loan = opening.dispatch.as_ref().unwrap().payload.clone();
    let mut budget: IrlPipeBudget = fixture(|_| true);
    budget.object_bytes = loan.object.byte_length;
    let mut call = PipeCall::new(
        opening.clone(),
        IrlPipeLink::IRL_CORE_WORKER,
        Some(&binding),
        &budget,
    )
    .unwrap();
    apply(&mut call, ToCallee, &opening).unwrap();
    let mut request = frame(IRL_FETCH);
    request.borrow = Some(loan.clone());
    assert!(apply(&mut call, ToCaller, &request).is_err());
    request.call = opening.call;
    assert!(apply(&mut call, ToCallee, &request).is_err());
    apply(&mut call, ToCaller, &request).unwrap();
    assert!(
        call.fetch(&loan, |_, _| Err("outside visible frontier".into()), LIMITS)
            .is_err()
    );
    let good = fixture::<IrlMaterial>(|v| {
        v["object"]["digest"] == core_types::scalar::hex(&loan.object.digest.0)
    });
    let mut corrupt = good.data.0.clone();
    corrupt[0] ^= 1;
    assert!(call.fetch(&loan, |_, _| Ok(corrupt), LIMITS).is_err());
    let result = call
        .fetch(
            &loan,
            |prefix, reference| {
                assert_eq!((prefix, reference), (&binding, &loan.object));
                Ok(good.data.0.clone())
            },
            LIMITS,
        )
        .unwrap();
    assert_eq!(result, good);
    assert!(call.fetch(&loan, |_, _| unreachable!(), LIMITS).is_err());
    assert!(apply(&mut call, ToCaller, &request).is_err());
}

#[test]
fn requesting_cancellation_allows_racing_completion_but_effective_cancellation_closes_call() {
    use Direction::*;
    use IrlPipeOperation::*;
    let (opening, binding) = core_opening();
    let budget = fixture(|_| true);
    for effective in [false, true] {
        let mut call = PipeCall::new(
            opening.clone(),
            IrlPipeLink::IRL_CORE_WORKER,
            Some(&binding),
            &budget,
        )
        .unwrap();
        apply(&mut call, ToCallee, &opening).unwrap();
        let correlated = |op| IrlPipeFrame {
            call: opening.call.clone(),
            ..frame(op)
        };
        apply(&mut call, ToCallee, &correlated(IRL_CANCEL_REQUEST)).unwrap();
        if effective {
            apply(&mut call, ToCaller, &correlated(IRL_CANCEL_EFFECTIVE)).unwrap();
        }
        assert_eq!(
            apply(&mut call, ToCaller, &correlated(IRL_CANDIDATE)).is_ok(),
            !effective
        );
    }
}

#[test]
fn endpoints_require_current_delegation_and_unknown_outcome_cannot_restart_call() {
    use Direction::*;
    use IrlPipeLink::*;
    use IrlPipeOperation::*;
    let budget: IrlPipeBudget = fixture(|_| true);
    assert!(frame_length(budget.frame_bytes, &budget).is_ok());
    assert!(frame_length(budget.frame_bytes + 1, &budget).is_err());
    for (link, end) in [
        (IRL_WORKER_BRIDGE, IRL_NOT_SENT),
        (IRL_WORKER_BRIDGE, IRL_SENT_UNKNOWN),
        (IRL_WORKER_ADAPTER, IRL_OUTPUT),
    ] {
        let mut opening = frame(IRL_DELEGATE);
        let d: IrlDelegation = fixture(|v| v["link"] == u32::from(link));
        let permitted: Vec<_> = d.inputs.iter().map(|m| m.object.clone()).collect();
        opening.delegation = Some(d);
        let mut call = PipeCall::new(opening.clone(), link, None, &budget).unwrap();
        assert!(apply(&mut call, ToCallee, &opening).is_err());
        assert!(
            call.receive(
                ToCaller,
                &opening.protobuf_bytes(LIMITS).unwrap(),
                LIMITS,
                |_| Ok(&permitted)
            )
            .is_err()
        );
        call.receive(
            ToCallee,
            &opening.protobuf_bytes(LIMITS).unwrap(),
            LIMITS,
            |_| Ok(&permitted),
        )
        .unwrap();
        let result = IrlPipeFrame {
            call: opening.call.clone(),
            ..frame(end)
        };
        assert_eq!(apply(&mut call, ToCaller, &result).unwrap(), result);
        assert!(
            call.receive(
                ToCallee,
                &opening.protobuf_bytes(LIMITS).unwrap(),
                LIMITS,
                |_| Ok(&permitted)
            )
            .is_err()
        );
    }
}
