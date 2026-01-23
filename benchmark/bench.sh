#!/bin/bash
mkdir -p results

python bench_create_table.py
python bench_insert.py
python bench_scan.py
python bench_read_row.py
python bench_insert_2.py
python bench_scan_2.py
python bench_read_row_2.py
python plot_results.py
