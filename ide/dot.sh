#!/bin/bash

tail -n +6 $1 > /tmp/graph.dot
dot -Tsvg /tmp/graph.dot -o /tmp/graph.svg
firefox /tmp/graph.svg
