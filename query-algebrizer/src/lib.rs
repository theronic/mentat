// Copyright 2016 Mozilla
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0
// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

#[macro_use]
extern crate error_chain;

extern crate mentat_core;
extern crate mentat_query;

use std::collections::BTreeSet;

mod errors;
mod types;
mod validate;
mod clauses;

use mentat_core::{
    Schema,
};

use mentat_core::counter::RcCounter;

use mentat_query::{
    FindQuery,
    FindSpec,
    SrcVar,
    Variable,
};

pub use errors::{
    Error,
    ErrorKind,
    Result,
};

#[allow(dead_code)]
pub struct AlgebraicQuery {
    default_source: SrcVar,
    pub find_spec: FindSpec,
    pub with: BTreeSet<Variable>,
    has_aggregates: bool,
    pub limit: Option<u64>,
    pub cc: clauses::ConjoiningClauses,
}

impl AlgebraicQuery {
    /**
     * Apply a new limit to this query, if one is provided and any existing limit is larger.
     */
    pub fn apply_limit(&mut self, limit: Option<u64>) {
        match self.limit {
            None => self.limit = limit,
            Some(existing) =>
                match limit {
                    None => (),
                    Some(new) =>
                        if new < existing {
                            self.limit = limit;
                        },
                },
        };
    }

    #[inline]
    pub fn is_known_empty(&self) -> bool {
        self.cc.is_known_empty()
    }
}

pub fn algebrize_with_counter(schema: &Schema, parsed: FindQuery, counter: usize) -> Result<AlgebraicQuery> {
    let alias_counter = RcCounter::with_initial(counter);
    let cc = clauses::ConjoiningClauses::with_alias_counter(alias_counter);
    algebrize_with_cc(schema, parsed, cc)
}

pub fn algebrize(schema: &Schema, parsed: FindQuery) -> Result<AlgebraicQuery> {
    algebrize_with_cc(schema, parsed, clauses::ConjoiningClauses::default())
}

#[allow(dead_code)]
pub fn algebrize_with_cc(schema: &Schema, parsed: FindQuery, mut cc: ConjoiningClauses) -> Result<AlgebraicQuery> {
    // TODO: integrate default source into pattern processing.
    // TODO: flesh out the rest of find-into-context.
    let where_clauses = parsed.where_clauses;
    for where_clause in where_clauses {
        cc.apply_clause(schema, where_clause)?;
    }
    cc.expand_column_bindings();
    cc.prune_extracted_types();

    let limit = if parsed.find_spec.is_unit_limited() { Some(1) } else { None };
    Ok(AlgebraicQuery {
        default_source: parsed.default_source,
        find_spec: parsed.find_spec,
        has_aggregates: false,           // TODO: we don't parse them yet.
        with: parsed.with.into_iter().collect(),
        limit: limit,
        cc: cc,
    })
}

pub use clauses::{
    ConjoiningClauses,
};

pub use types::{
    Column,
    ColumnAlternation,
    ColumnConstraint,
    ColumnConstraintOrAlternation,
    ColumnIntersection,
    ColumnName,
    ComputedTable,
    DatomsColumn,
    DatomsTable,
    QualifiedAlias,
    QueryValue,
    SourceAlias,
    TableAlias,
    VariableColumn,
};

