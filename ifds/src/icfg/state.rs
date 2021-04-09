use crate::icfg::graph::*;
use crate::ir::ast::Function as AstFunction;
use crate::ir::ast::Instruction;

use crate::{counter::Counter, solver::Request};
use anyhow::{anyhow, bail, Context, Result};

use log::debug;

use std::collections::HashMap;

type FunctionName = String;

#[derive(Debug, Default)]
pub struct State {
    facts: HashMap<FunctionName, Vec<Fact>>,
    pub vars: HashMap<FunctionName, Vec<Variable>>,
    pub functions: HashMap<FunctionName, Function>,
    /// `init_facts` is a helper struct for getting the initial facts
    /// of a functions. We need this because we have to reinitalize the
    /// function when function is calling itself.
    init_facts: HashMap<FunctionName, Vec<Fact>>,
    note_counter: Counter,
    pub notes: Vec<Note>,
}

impl State {
    pub fn get_taut(&self, function: &String) -> Result<Option<&Fact>> {
        /*
        self.edges
            .iter()
            .find(|x| x.get_from().var_is_taut && &x.get_from().function == function)
            .map(|x| x.get_from())*/

        Ok(self
            .facts
            .get(function)
            .context("Cannot find function")?
            .iter()
            .find(|x| x.var_is_taut))
    }

    pub fn is_function_defined(&self, name: &String) -> bool {
        self.functions.get(name).is_some()
    }

    /// Saving facts into an internal structure for fast lookup.
    fn cache_facts(&mut self, function: &String, facts: Vec<Fact>) -> Result<&[Fact]> {
        let mut reference: Option<&[Fact]> = None;

        use std::collections::hash_map::Entry;

        match self.facts.entry(function.clone()) {
            Entry::Occupied(entry) => {
                let saver = entry.into_mut();
                let len1 = saver.len();
                saver.extend_from_slice(facts.as_slice());
                let len2 = saver.len();

                reference = Some(&saver[len1..len2]);
            }
            Entry::Vacant(entry) => {
                reference = Some(entry.insert(facts));
            }
        }

        Ok(reference.unwrap())
    }

    pub fn cache_fact(&mut self, function: &String, fact: Fact) -> Result<&Fact> {
        let v = vec![fact];
        let res = self.cache_facts(function, v)?;
        Ok(&res[0])
    }

    fn init_function_def(&mut self, function: &AstFunction) -> Result<()> {
        self.functions.insert(
            function.name.clone(),
            Function {
                name: function.name.clone(),
                definitions: function.definitions.len(),
                return_count: function.results_len,
            },
        );

        Ok(())
    }

    /// Add a memory variable to the graph's variables
    pub fn add_memory_var(&mut self, variable: String, function: String, offset: f64) -> Variable {
        let name = format!("{}@{}", variable, offset);
        let var = Variable {
            function: function.clone(),
            is_global: false,
            is_memory: true,
            is_taut: false,
            name,
            memory_offset: Some(offset),
        };

        if let Some(vars) = self.vars.get_mut(&function) {
            if !vars.contains(&var) {
                vars.push(var.clone());
            }
        } else {
            self.vars.insert(function, vec![var.clone()]);
        }

        var
    }

    /// Initialise the function.
    pub fn init_function(&mut self, function: &AstFunction, pc: usize) -> Result<Vec<Fact>> {
        debug!("Adding new function {} to the graph", function.name);

        if let Some(function) = self.functions.get(&function.name) {
            debug!("Function was already initialized");

            // Handle edge case when the smallest fact greater than `pc`.
            // This might happen if you the user starts the analysis from not `0`,
            // but there is a self recursive call.

            let min_pc = self
                .facts
                .get(&function.name)
                .context("Cannot find function")?
                .iter()
                .map(|x| x.next_pc)
                .min()
                .context("No facts found")?;

            if min_pc <= pc {
                // no self recursion

                // Return the first facts of the function.
                return Ok(self
                    .init_facts
                    .get(&function.name)
                    .context("Cannot get function's stored init facts")?
                    .clone());
            }

            // else reinitalize the function.
        }

        self.init_function_def(function)?;

        let mut variables = Vec::with_capacity(function.definitions.len() + 1);

        variables.push(Variable {
            name: "taut".to_string(),
            function: function.name.clone(),
            is_global: false,
            is_taut: true,
            is_memory: false,
            memory_offset: None,
        });

        // add definitions
        for reg in function.definitions.iter() {
            debug!("Adding definition {}", reg);

            let reg_num: isize = reg
                .clone()
                .split_off(1)
                .parse()
                .context("Cannot parse reg to number")?;
            let is_global = match reg_num {
                x if x < 0 => true,
                x if x >= 0 => false,
                _ => unreachable!(""),
            };

            variables.push(Variable {
                name: reg.clone(),
                function: function.name.clone(),
                is_global,
                is_taut: false,
                is_memory: false,
                memory_offset: None,
            });
        }

        let facts = self
            .init_facts(function, &mut variables, pc)
            .context("Cannot initialize facts")?;

        self.vars.insert(function.name.clone(), variables);

        // insert the initial facts or update them.
        self.init_facts.insert(function.name.clone(), facts.clone());

        Ok(facts)
    }

    fn init_facts(
        &mut self,
        function: &AstFunction,
        variables: &mut Vec<Variable>,
        pc: usize,
    ) -> Result<Vec<Fact>> {
        debug!("Initializing facts for function {}", function.name);

        let mut index = 0;
        let mut facts = Vec::with_capacity(variables.len());
        for var in variables {
            debug!("Creating fact for var {}", var.name);

            let fact = Fact {
                belongs_to_var: var.name.clone(),
                var_is_global: var.is_global,
                var_is_taut: var.is_taut,
                var_is_memory: var.is_memory,
                next_pc: pc,
                track: index,
                function: function.name.clone(),
                memory_offset: var.memory_offset,
            };

            //self.facts.push(fact.clone());
            facts.push(fact);

            index += 1;
        }

        Ok(facts)
    }

    /// Get the facts at a certain instruction for a given function.
    pub fn get_facts_at(
        &self,
        function: &String,
        pc: usize,
    ) -> Result<impl Iterator<Item = &Fact>> {
        Ok(self
            .facts
            .get(function)
            .context("Cannot find function")?
            .iter()
            .filter(move |x| x.next_pc == pc))
    }

    /// Get the track by given name and function.
    pub fn get_track(&self, function: &String, variable: &String) -> Option<usize> {
        self.vars
            .get(function)?
            .iter()
            .position(|x| &x.name == variable)
    }

    /// Get a variable by given name and function.
    pub fn get_var(&self, function: &String, variable: &String) -> Option<&Variable> {
        self.vars
            .get(function)?
            .iter()
            .find(|x| &x.name == variable)
    }

    /// Add a new statement to the graph.
    pub fn add_statement(
        &mut self,
        function: &AstFunction,
        instruction: String,
        pc: usize,
        variable: &String,
    ) -> Result<Vec<Fact>> {
        debug!(
            "Adding statement {} at {} for {} ({})",
            instruction, pc, variable, function.name
        );

        let vars = self
            .vars
            .get(&function.name)
            .context("Cannot get functions's vars")?
            .clone();
        let mut vars = vars.iter().enumerate();

        let mut facts = Vec::new();

        for (track, var) in vars.find(|x| &x.1.name == variable) {
            debug!("Adding new fact for {}", var.name);

            facts.push(Fact {
                belongs_to_var: var.name.clone(),
                var_is_global: var.is_global,
                var_is_taut: var.is_taut,
                var_is_memory: var.is_memory,
                track,
                function: function.name.clone(),
                next_pc: pc,
                memory_offset: var.memory_offset,
            });
        }

        Ok(facts)
    }

    /// Add a statement with the instruction with a note [`Note`].
    /// The notes has the instruction as content, which makes it easier to read in the
    /// `tikz` representation.
    pub fn add_statement_with_note(
        &mut self,
        function: &AstFunction,
        instruction: String,
        pc: usize,
        variable: &String,
    ) -> Result<Vec<Fact>> {
        let facts = self
            .add_statement(function, instruction.clone(), pc, variable)
            .context("While add statement with note")?;

        let vars = self
            .vars
            .get(&function.name)
            .context("Cannot get functions's vars")?
            .clone();
        let mut vars = vars.iter().enumerate();

        for (_track, var) in vars.find(|x| &x.1.name == variable) {
            debug!("Adding new fact for {}", var.name);

            if var.is_taut && pc < function.instructions.len() {
                self.notes.push(Note {
                    id: self.note_counter.get(),
                    function: function.name.clone(),
                    pc,
                    note: instruction.clone(),
                });
            }
        }

        Ok(facts)
    }

    /// Create new tautological fact by given function and pc.
    pub fn taut(&mut self, function: String, pc: usize) -> Fact {
        Fact {
            belongs_to_var: "taut".to_string(),
            var_is_taut: true,
            var_is_global: false,
            var_is_memory: false,
            function,
            next_pc: pc,
            track: 0,
            memory_offset: None,
        }
    }
}
