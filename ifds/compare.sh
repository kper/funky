#!/bin/bash

rm /tmp/graph.tex
rm /tmp/graph.pdf
rm /tmp/graph2.tex
rm /tmp/graph2.pdf
tail -n +6 $1 > /tmp/graph.tex
tail -n +6 $1.new > /tmp/graph2.tex
xelatex /tmp/graph.tex 
xelatex /tmp/graph2.tex 
evince graph.pdf &
evince graph2.pdf
