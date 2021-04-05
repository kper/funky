#!/bin/bash

rm /tmp/graph.tex
rm /tmp/graph.pdf
cp $1 /tmp/graph.tex
xelatex /tmp/graph.tex && evince graph.pdf
