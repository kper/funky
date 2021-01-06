#!flask/bin/python

from flask import Flask, request, render_template, redirect
import sqlite3 as sql
from flask_sqlalchemy import SQLAlchemy
from datetime import datetime

app = Flask(__name__)
app.config ['SQLALCHEMY_DATABASE_URI'] = 'sqlite:///database.sqlite3'

db = SQLAlchemy(app)

class perf_run(db.Model):
    __tablename__ = 'perf_run'


    #{'commit': b'166cfcdac03efa6bd96523e2b252a9aec477302e', 'cache-misses': '604318', 'branch-misses': '1420790', 'cpu-cycles': '336041733', 'instructions': '332257074', 'branch-instructions': '63355474'}
    id = db.Column('id', db.Integer, primary_key = True)
    commit = db.Column('commit', db.String(64))
    cache_misses = db.Column('cache-misses', db.BigInteger)
    branch_misses = db.Column('branch-misses', db.BigInteger)
    cpu_cycles = db.Column('cpu-cycles', db.BigInteger)
    instructions = db.Column('instructions', db.BigInteger)
    branch_instructions = db.Column('branch-instructions', db.BigInteger)
    on_create = db.Column('on-create', db.DateTime)

    def __init__(self, commit, cache_misses, branch_misses, cpu_cycles, instructions, branch_instructions, on_create):
        self.commit = commit
        self.cache_misses = cache_misses
        self.cpu_cycles = cpu_cycles
        self.instructions = instructions
        self.branch_instructions = branch_instructions
        self.on_create = on_create 

@app.route('/')
def index():
    return redirect('/perfrun')

@app.route('/perfrun', methods=['GET', 'POST'])
def test_run():
    if request.method == 'POST':
        data = request.get_json()
        print(data['cache-misses'])
        db.session.add(perf_run(commit=data['commit'], cache_misses = data['cache-misses'], branch_misses = data['branch-misses'], cpu_cycles = data['cpu-cycles'], instructions = data['instructions'], branch_instructions = data['branch-instructions'], on_create=datetime.now()))
        db.session.commit()
        return 'Ok'
    elif request.method == 'GET':
        rows = perf_run.query.all()  
        print("rows", len(rows))
        return render_template("./perf.html", rows=rows)
