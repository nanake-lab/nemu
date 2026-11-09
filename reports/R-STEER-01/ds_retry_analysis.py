"""Frozen C1 semantic gates; real-data application requires the random20 review."""
from __future__ import annotations

from collections import defaultdict
from typing import Any

from ds_retry_prepare import SESSION
from ds_protocol import COHERENCE, CONCEPTS, HERE, load, sha, write


def at_least(numerator: int, denominator: int, percent: int) -> bool:
    return denominator > 0 and 100 * numerator >= percent * denominator


def coherent(rating: dict[str, Any]) -> bool:
    return all(rating['coherence'][name]['label'] == 'no' for name in COHERENCE)


def absence(rating: dict[str, Any]) -> bool:
    return all(rating['concepts'][name]['label'] == 'absent' for name in CONCEPTS)


def score(label: str) -> int:
    return 1 if label == 'positive' else -1 if label == 'negative' else 0


def metrics(keys: list[dict[str, Any]], ratings: dict[str, dict[str, Any]]) -> dict[str, Any]:
    # C1-BLIND-RUBRIC, four existing gates; repeat=1 is not an independent scene.
    base = [row for row in keys if row['repeat'] == 0]
    by_job = {row['job_id']: row for row in base}
    gates: dict[str, Any] = {}
    for lang in ('zh', 'en', 'ja'):
        rows = [r for r in base if r['lang'] == lang]
        targets = [r for r in rows if r['kind'] == 'target']
        success = [r for r in targets if r['id'] in ratings and ratings[r['id']]['concepts'][r['concept']]['label'] == ('positive' if r['expected_sign'] == 1 else 'negative')]
        faithful_coherent = sum(r['id'] in ratings and ratings[r['id']]['facts']['label'] == 'faithful' and coherent(ratings[r['id']]) for r in targets)
        per_concept: dict[str, Any] = {}
        for concept in CONCEPTS:
            cell = [r for r in targets if r['concept'] == concept]
            hits = sum(r in success for r in cell)
            per_concept[concept] = {'n': len(cell), 'hits': hits, 'passed': hits >= 14}
        generic = [r for r in rows if r['kind'] == 'generic']
        no_concept = sum(r['id'] in ratings and absence(ratings[r['id']]) for r in generic)
        facts = [r for r in rows if r['kind'] == 'fact']
        correct_facts = sum(r['id'] in ratings and ratings[r['id']]['facts']['label'] == ('faithful' if r['expected_sign'] == 1 else 'contradicted') for r in facts)
        free = [r for r in rows if r['kind'] == 'free']
        # Gate 4 literally specifies faithful / no contradiction, not a new all-five filter.
        faithful_uncontradicted = sum(r['id'] in ratings and ratings[r['id']]['facts']['label'] == 'faithful' and ratings[r['id']]['coherence']['contradiction']['label'] == 'no' for r in free)
        all_five = sum(r['id'] in ratings and ratings[r['id']]['facts']['label'] == 'faithful' and coherent(ratings[r['id']]) for r in free)
        signs: dict[str, Any] = {}
        for concept in CONCEPTS:
            roots = sorted({r['root'] for r in free if r['concept'] == concept})
            root_values: list[dict[str, Any]] = []
            for root in roots:
                levels: dict[str, dict[str, Any]] = {}
                for level in (-1, 0, 1):
                    cell = sorted((r for r in free if r['root'] == root and r['level'] == level), key=lambda r: r['order'])
                    values = [score(ratings[r['id']]['concepts'][concept]['label']) for r in cell if r['id'] in ratings]
                    bad = [r['id'] for r in cell if r['id'] not in ratings]
                    levels[str(level)] = {'ids': [r['id'] for r in cell], 'scores': values,
                                          'invalid_ids': bad, 'mean': None if bad else sum(values) / len(values),
                                          'uncertain': sum(r['id'] in ratings and ratings[r['id']]['concepts'][concept]['label'] == 'uncertain' for r in cell)}
                identifiable = all(value['mean'] is not None for value in levels.values())
                root_values.append({'root': root, 'levels': levels,
                                    'delta': levels['1']['mean'] - levels['-1']['mean'] if identifiable else None})
            delta = None if any(r['delta'] is None for r in root_values) else sum(r['delta'] for r in root_values) / len(root_values)
            signs[concept] = {'roots': root_values, 'equal_root_mean_delta': delta, 'passed': delta is not None and delta >= 0.5}
        gates[lang] = {
            'target': {'n': len(targets), 'correct_target_sign': len(success), 'faithful_and_coherent': faithful_coherent,
                       'per_concept': per_concept,
                       'passed': at_least(len(success), len(targets), 90) and at_least(faithful_coherent, len(targets), 90) and all(v['passed'] for v in per_concept.values())},
            'generic': {'n': len(generic), 'all_four_absent': no_concept, 'passed': at_least(no_concept, len(generic), 75)},
            'facts': {'n': len(facts), 'correct': correct_facts, 'passed': correct_facts >= 14},
            'free': {'n': len(free), 'faithful_no_contradiction': faithful_uncontradicted,
                     'faithful_all_five_coherence_no_diagnostic': all_five, 'signed_effects': signs,
                     'passed': at_least(faithful_uncontradicted, len(free), 90) and all(v['passed'] for v in signs.values())}}
    pairs = []
    for repeat in (r for r in keys if r['repeat'] == 1):
        original = by_job[repeat['job_id']]
        valid = original['id'] in ratings and repeat['id'] in ratings
        before, after = ratings.get(original['id'], {}), ratings.get(repeat['id'], {})
        pairs.append({'id': repeat['id'], 'original_id': original['id'], 'lang': repeat['lang'],
                      'concept_vector': valid and all(before['concepts'][c]['label'] == after['concepts'][c]['label'] for c in CONCEPTS),
                      'facts': valid and before['facts']['label'] == after['facts']['label'],
                      'coherence_vector': valid and all(before['coherence'][c]['label'] == after['coherence'][c]['label'] for c in COHERENCE)})
    repeat_summary: dict[str, Any] = {}
    for group in ('all', 'zh', 'en', 'ja'):
        cell = pairs if group == 'all' else [p for p in pairs if p['lang'] == group]
        limit = 85 if group == 'all' else 80
        counts = {name: sum(p[name] for p in cell) for name in ('concept_vector', 'facts', 'coherence_vector')}
        repeat_summary[group] = {'n': len(cell), 'exact_counts': counts,
                                 'passed': all(at_least(n, len(cell), limit) for n in counts.values())}
    label_counts: dict[str, dict[str, int]] = defaultdict(lambda: defaultdict(int))
    for row in base:
        if row['id'] not in ratings:
            label_counts[row['lang'] + ':invalid']['whole_item'] += 1
            continue
        rating = ratings[row['id']]
        for concept in CONCEPTS:
            label_counts[row['lang'] + ':' + concept][rating['concepts'][concept]['label']] += 1
    passed = all(v['passed'] for lang in gates.values() for v in lang.values()) and all(v['passed'] for v in repeat_summary.values())
    return {'semantic_decision': 'C1_semantic_pass' if passed else 'C1_semantic_insufficient_stop',
            'automatic_decision_unchanged': 'C1_insufficient_stop', 'base_units': len(base), 'repeat_units': len(pairs),
            'language_gates': gates, 'repeat_summary': repeat_summary, 'repeat_pairs': pairs,
            'base_label_counts': dict(label_counts), 'pilot_scores_used': False}


def analyze() -> None:
    plan = load(SESSION / 'plan.json')
    result = load(SESSION / 'run/result.json')
    if not result['collection_complete']:
        raise ValueError('504 complete valid-or-invalid collection required')
    approval = load(SESSION / 'sample-review.json')
    if approval.get('approved') is not True or approval.get('sample_ids') != plan['sample_ids']:
        raise ValueError('Explicit user review of fixed random20 required before formal use')
    inventory_path = SESSION / 'formal-input-inventory.json'
    if approval.get('formal_input_inventory_sha256') != sha(inventory_path):
        raise ValueError('User-reviewed formal input inventory differs')
    inventory = load(inventory_path)
    if [r['id'] for r in inventory] != [r['id'] for r in plan['requests']]:
        raise ValueError('Only the new 504 may enter formal analysis')
    for row in inventory:
        for name, digest in row['artifacts'].items():
            if sha(SESSION / 'run' / row['id'] / name) != digest:
                raise ValueError('Formal data changed after publication')
    # No pilot rating source is read; every formal item comes from this new full run.
    keys = load(HERE / 'results/c1-blind-key-02.json')
    ratings = {row['id']: load(SESSION / 'run' / row['id'] / 'rating.json') for row in inventory if row['outcome'] == 'rating'}
    value = metrics(keys, ratings)
    value['plan_sha256'] = sha(SESSION / 'plan.json')
    value['review_approval'] = approval
    write(SESSION / 'formal-analysis.json', value)


if __name__ == '__main__':
    analyze()
