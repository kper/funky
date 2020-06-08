#!/usr/bin/env python3

import json
import sys
import os
import subprocess


module_file = ''
verbose = False

path = sys.argv[1]
rootdir = '.'

binary = 'target/debug/funky'

cases = []

failures = []
successes = 0


def format_val(arg):
    return "{}({})".format(arg['type'].upper(), arg['value'])

def format_output(val):
    return 'Some(Value({}))'.format(val)

if os.path.dirname(path) != '':
    rootdir = os.path.dirname(path)

with open(path, "r") as read_file:
    data = json.load(read_file)
    for command in data['commands']:
        if command['type'] == 'module':
            module_file = command['filename']
            module_file = rootdir + '/' + module_file
            if verbose:
                print('[*] Found module file', module_file)
        if command['type'] == 'assert_return' and len(command['expected']) > 0:
            if 'args' not in command['action']:
                command['action']['args'] = []
            cases.append({
                'args': list(map(lambda x: format_val(x), command['action']['args'])),
                # force index 0 here since the current wasm standard only alows for one return element
                'expected': format_val(command['expected'][0]),
                'name': command['action']['field']
            })

for case in cases:
    out = subprocess.run([binary,module_file,case['name'],' '.join(case['args']),'--spec'],stdout=subprocess.PIPE,stderr=subprocess.PIPE)
    result = out.stdout.decode("utf-8").rstrip()
    if verbose:
        print(f'--- Testcase {case["name"]} ---')
    if result != format_output(case['expected']):
        failures.append(case)
        if verbose:
            print('[!] Assertion failed: {} != {}'.format(result, format_output(case['expected'])))
            if out.stderr:
                print('[!] Encountered error:')
                print(out.stderr.decode('utf-8'))
    else:
        successes += 1
        if verbose:
            print('[*] Success')

print(f"--- {path} ---")
if len(cases) > 0:
    print(f"Success: {successes}")
    print(f"Failures: {len(failures)}")
    print(f"Success rate: {((successes/len(cases))*100):.2f}%")
else:
    print("No testcases found")
