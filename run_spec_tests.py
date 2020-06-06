#!/usr/bin/env python3

import json
import sys
import os


module_file = ''

path = sys.argv[1]
rootdir = '.'

binary = 'target/debug/funky'

cases = []


def format_val(arg):
    return "'{}({})'".format(arg['type'].upper(), arg['value'])

if os.path.dirname(path) != '':
    rootdir = os.path.dirname(path)

with open(path, "r") as read_file:
    data = json.load(read_file)
    for command in data['commands']:
        if command['type'] == 'module':
            module_file = command['filename']
            module_file = rootdir + '/' + module_file
            print('[*] Found module file', module_file)
        if command['type'] == 'assert_return' and len(command['expected']) > 0:
            cases.append({
                'args': list(map(lambda x: format_val(x), command['action']['args'])),
                # force index 0 here since the current wasm standard only alows for one return element
                'expected': format_val(command['expected'][0]),
                'name': command['action']['field']
            })

for case in cases:
    os.system(' '.join([binary,module_file,case['name'],' '.join(case['args']),'--spec']))
