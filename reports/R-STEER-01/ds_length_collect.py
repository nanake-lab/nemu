"""Use unchanged review and metrics with the explicitly declared composite input."""
import argparse
from pathlib import Path

import ds_retry_analysis
import ds_retry_review
from ds_length_prepare import SESSION

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('mode', choices=('review', 'analysis'))
    parser.add_argument('--output', type=Path)
    args = parser.parse_args()
    if args.mode == 'review':
        if args.output is None:
            parser.error('--output required')
        vars(ds_retry_review)['SESSION'] = SESSION
        ds_retry_review.publish(args.output)
    else:
        vars(ds_retry_analysis)['SESSION'] = SESSION
        ds_retry_analysis.analyze()
