"""Fixed synthetic challenges. No production or personal-data imports."""
import random
from dataclasses import replace
from evidence import Criterion, Record, eligible, estimate, justified_order, predictions
from learning import Adaptive, Mean, models

Row = dict[str, str | int | float]


def e1(seed: int) -> list[Row]:
    rng = random.Random(seed)
    output: list[Row] = []
    for world in ("mar", "mnar"):
        data: list[list[Record]] = [[], []]
        for a in range(2):
            for t in range(400):
                p = (.9 if a == 0 else .6) if world == "mar" else (.4 if a == 0 else .6)
                y, visible = int(rng.random() < p), rng.random()
                seen = visible < (.3 if a == 0 else 1) if world == "mar" else bool(y or a == 1)
                data[a].append(Record(f"{a}-{t}", 0, t, "c", str(a), "goal", outcome=y if seen else None))
        zero = [sum(r.outcome or 0 for r in arm)/400 for arm in data]
        known_mar = [estimate(arm, mar=True) for arm in data]
        unknown = [estimate(arm) for arm in data]
        choices = {
            "zero_fill": int(zero[1] > zero[0]),
            "resolved_point": int(known_mar[1].forecast > known_mar[0].forecast),
            "declared_mar_interval": justified_order(*known_mar),
            "unknown_mechanism": justified_order(*unknown),
        }
        optimum = 0 if world == "mar" else 1
        for arm, choice in choices.items():
            output.append(dict(experiment=f"E1_{world}", arm=arm, seed=seed,
                               correct=int(choice == optimum), wrong=int(choice is not None and choice != optimum),
                               abstain=int(choice is None)))
    return output


def e2(seed: int) -> tuple[list[Row], list[Row]]:
    rng = random.Random(seed)
    tape = [rng.random() for _ in range(512)]
    output: list[Row] = []
    traces: list[Row] = []
    for world in ("stable", "abrupt", "gradual", "cycle"):
        for arm, model in models().items():
            mse = late_mse = wrong = 0.0
            delay, consecutive = 193, 0
            for t, uniform in enumerate(tape):
                if world == "stable":
                    p = .7
                elif world == "abrupt":
                    p = .7 if t < 320 else .3
                elif world == "gradual":
                    p = .7-.4*min(1, max(0, (t-192)/192))
                else:
                    p = .7 if (t//128) % 2 == 0 else .3
                prediction = model.predict()
                if t >= 64:
                    mse += (prediction-p)**2
                    wrong += int(prediction < .5)
                if t >= 320:
                    late_mse += (prediction-p)**2
                    consecutive = consecutive+1 if prediction < .5 else 0
                    if consecutive >= 5 and delay == 193:
                        delay = t-320+1
                y = int(uniform < p)
                model.update(y)
                if seed == 41000:
                    traces.append(dict(experiment=f"E2_{world}", arm=arm, t=t, truth=p, prediction=prediction, y=y))
            row: Row = dict(experiment=f"E2_{world}", arm=arm, seed=seed, mse=mse/448, late_mse=late_mse/192)
            if world == "stable":
                row["wrong_side"] = wrong/448
            if world == "abrupt":
                row.update(delay_capped=delay, censored=int(delay == 193))
            if isinstance(model, Adaptive):
                row["mean_hazard"] = model.mean_hazard()
                for i, w in enumerate(model.weights):
                    row[f"hazard_weight_{i}"] = w
            output.append(row)
    return output, traces


def e3(seed: int) -> list[Row]:
    rng = random.Random(seed)
    data: list[Record] = []
    goal = Criterion("task")
    for context in ("busy", "free"):
        for action in ("a", "b"):
            p = .85 if (context == "free") == (action == "a") else .15
            for _ in range(200):
                t = len(data)
                data.append(Record(str(t), 0, t, context, action, "task", outcome=int(rng.random() < p)))
    pooled = [estimate([r for r in data if r.action == a]) for a in ("a", "b")]
    context_accuracy = global_accuracy = migration_error = 0.0
    swapped = [replace(r, action="b" if r.action == "a" else "a") for r in data]
    for context in ("busy", "free"):
        scores = predictions(data, goal, context)
        transformed = predictions(swapped, goal, context)[::-1]
        optimum = 0 if context == "free" else 1
        context_accuracy += int((scores[1].forecast > scores[0].forecast) == bool(optimum))/2
        global_accuracy += int((pooled[1].forecast > pooled[0].forecast) == bool(optimum))/2
        migration_error = max(migration_error, *(abs(x.forecast-y.forecast) for x, y in zip(scores, transformed)))
    output: list[Row] = [dict(experiment="E3_context", arm=arm, seed=seed, accuracy=value)
                         for arm, value in (("pooled", global_accuracy), ("conditioned", context_accuracy), ("frozen", .5))]
    output.append(dict(experiment="E3_migration", arm="label_permutation", seed=seed, max_error=migration_error))
    outcomes: list[Record] = []
    for action, p, cost in (("a", .9, .9), ("b", .6, .1)):
        for _ in range(200):
            t = len(outcomes)
            outcomes.append(Record(str(t), 0, t, "c", action, "task", outcome=int(rng.random() < p), cost=cost))
    old, new = Criterion("task", cost_weight=.1), Criterion("task", cost_weight=.8)
    scores = predictions(outcomes, old, "c")
    for arm, criterion in (("old_cache", old), ("revalue", new)):
        chosen = int(scores[1].value(criterion) > scores[0].value(criterion))
        output.append(dict(experiment="E3_revalue", arm=arm, seed=seed, correct=chosen))
    assert len(eligible(outcomes, new)) == 400
    return output


def e4(seed: int) -> list[Row]:
    rng = random.Random(seed)
    tape = [[rng.random() for _ in range(4)] for _ in range(240)]
    model = [Mean(), Mean()]
    data: list[list[Record]] = [[], []]
    for t, u in enumerate(tape):
        if t < 8:
            a = t % 2
        elif u[0] < .1 or model[0].predict() == model[1].predict():
            a = int(u[1] >= .5)
        else:
            a = int(model[1].predict() > model[0].predict())
        y = int(u[2+a] < .7)
        model[a].update(y)
        data[a].append(Record(str(t), 0, t, "c", str(a), "task", outcome=y))
    estimates = [estimate(arm, mar=True) for arm in data]
    output: list[Row] = [dict(experiment="E4_equal_actions", arm="learn", seed=seed,
                             absolute_bias=abs(len(data[0])-len(data[1]))/240,
                             predicted_gap=abs(model[0].predict()-model[1].predict()),
                             false_justified_order=int(justified_order(*estimates) is not None))]
    audit: list[list[Record]] = [[], []]
    for a in range(2):
        for t in range(400):
            audit[a].append(Record(str(t), 0, t, "c", str(a), "task", outcome=int(rng.random() < .7)))
    audited = [estimate(arm, mar=True) for arm in audit]
    output.append(dict(experiment="E4_equal_actions", arm="independent_audit", seed=seed,
                       predicted_gap=abs(audited[0].forecast-audited[1].forecast),
                       false_justified_order=int(justified_order(*audited) is not None)))
    return output
