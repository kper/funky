#!/usr/bin/env python3

import json
import sys
import os
import subprocess
import logging
import csv


module_file = ''

path = sys.argv[1]
rootdir = '.'
verbose = False

binary = 'target/debug/funky'

cases = []

failures = []
successes = 0

def force_signed(val, bits):
    return (val & ((1 << bits-1) - 1)) - (val & (1 << bits-1))


def format_val(arg):
    if arg['type'] == 'i32':
        val = force_signed(int(arg['value']),32)
        return f"I32({val})"
    if arg['type'] == 'i64':
        val = force_signed(int(arg['value']),64)
        return f"I64({val})"
    if arg['type'] == 'f32':
        if arg['value'][0:3] == 'nan':
            return "{}(NaN)".format(arg['type'].upper())
        if arg['value'] == '4286578688':
            return "F32(-inf)"
        if arg['value'] == '2139095040':
            return "F32(inf)"
        return "{}({:.1f})".format(arg['type'].upper(), float(arg['value']))
    if arg['type'] == 'f64':
        if arg['value'][0:3] == 'nan':
            return "{}(NaN)".format(arg['type'].upper())
        if arg['value'] == '18442240474082181120':
            return "F64(-inf)"
        if arg['value'] == '9218868437227405312':
            return "F64(inf)"
        return "{}({:.1f})".format(arg['type'].upper(), float(arg['value']))

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
            logging.info('Found module file %s',module_file)
        if command['type'] == 'assert_return' and len(command['expected']) > 0:
            if 'args' not in command['action']:
                command['action']['args'] = []
            cases.append({
                'args': list(map(lambda x: format_val(x), command['action']['args'])),
                # force index 0 here since the current wasm standard only alows for one return element
                'expected': format_val(command['expected'][0]),
                'name': command['action']['field'],
                'line': command['line']
            })

idx = 1
reportfile = open('report.csv','a')
reportwriter = csv.writer(reportfile)
for case in cases:
    args = [binary,module_file,case['name']] + case['args'] + ['--spec']
    out = subprocess.run(args,stdout=subprocess.PIPE,stderr=subprocess.PIPE)
    result = out.stdout.decode("utf-8").rstrip()
    if result != format_output(case['expected']):
        failures.append(case)
        print(f"[FAILED]: {case['name']}({' '.join(case['args'])}) @{case['line']}")
        print('[FAILED]: Assertion failed!')
        print(f'[FAILED]: Expected:\t{format_output(case["expected"])}')
        print(f'[FAILED]: Actual:\t{result}')
        if out.stderr:
            print('\tEncountered error:')
            print(out.stderr.decode('utf-8'))
        reportwriter.writerow([path,'FAIL',case['name'],' '.join(case['args'])])
    else:
        reportwriter.writerow([path,'OK',case['name'],' '.join(case['args'])])
        if verbose:
            print(f"[OK]: {case['name']}({' '.join(case['args'])}) ")
        successes += 1
    idx += 1
reportfile.close()

print(f"--- {path} ---")
if len(cases) > 0:
    print(f"Success: {successes}")
    print(f"Failures: {len(failures)}")
    print(f"Success rate: {((successes/len(cases))*100):.2f}%")
else:
    print("No testcases found")
