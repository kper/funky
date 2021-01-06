#!/usr/bin/python3

import csv
import subprocess
import requests
import sys

commit = subprocess.check_output(["git", "rev-parse", "--short", "HEAD"]).strip()

if len(sys.argv) == 2:
    sys.exit("You need to pass on a executable path and function")

with open('wolkentreiber.csv', 'r') as csv_file:
    reader = csv.reader(csv_file, delimiter=';')

    next(reader, None) # skip first two rows
    next(reader, None)

    payload = dict()
    payload['commit'] = commit
    payload['path'] = sys.argv[1] + '#' + sys.argv[2]

    for row in reader:
        value=row[0]
        name=row[2]

        payload[name] = value

    print(payload)

    req = requests.post('http://localhost:5000/perfrun', json=payload)
