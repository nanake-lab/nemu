"""Explicit collection-path adapter; original review and metric functions unchanged."""
from __future__ import annotations

import argparse
from pathlib import Path

import ds_full2_analysis
import ds_full2_review
from ds_full2_windowless_prepare import SESSION


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('mode', choices=('review', 'analysis'))
    parser.add_argument('--output', type=Path)
    args = parser.parse_args()
    if args.mode == 'review':
        if args.output is None:
            parser.error('--output is required for review')
        vars(ds_full2_review)['SESSION'] = SESSION
        ds_full2_review.publish(args.output)
    else:
        vars(ds_full2_analysis)['SESSION'] = SESSION
        ds_full2_analysis.analyze()
