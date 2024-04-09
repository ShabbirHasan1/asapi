// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------
/// Filtros/Estructuras auxiliares que podemos necesitar para Find y derivados.
use std::fmt::{self, Display};

#[derive(PartialEq, Clone, Debug)]
pub enum MongoOperator {
    EQ,
    NEQ,
    IN,
    NIN,
    GT,
    GTE,
    LT,
    LTE,
    Exists,
    HasType,
    ArrayContainsAll,
    Regex,
    // Lógicos
    AND,
    OR,
    NOT,
    NOR,
}

impl MongoOperator {
    pub fn variants() -> &'static [MongoOperator] {
        static VARIANTS: [MongoOperator; 12] = [
            MongoOperator::EQ,
            MongoOperator::NEQ,
            MongoOperator::Exists,
            MongoOperator::IN,
            MongoOperator::NIN,
            MongoOperator::HasType,
            MongoOperator::ArrayContainsAll,
            MongoOperator::GT,
            MongoOperator::GTE,
            MongoOperator::LT,
            MongoOperator::LTE,
            MongoOperator::Regex,
        ];
        &VARIANTS
    }

    pub fn as_str(&self) -> &str {
        match self {
            MongoOperator::EQ => "Equals",
            MongoOperator::NEQ => "Doesn't Equal",
            MongoOperator::Exists => "Exists",
            MongoOperator::IN => "In",
            MongoOperator::NIN => "Not In",
            MongoOperator::HasType => "Has Type",
            MongoOperator::ArrayContainsAll => "Array Contains All",
            MongoOperator::GT => ">",
            MongoOperator::GTE => ">=",
            MongoOperator::LT => "<",
            MongoOperator::LTE => "<=",
            MongoOperator::Regex => "Regex",
            _ => "",
        }
    }

    pub fn as_mongo_operator(&self) -> &str {
        extract_operator(self)
    }
}

impl Display for MongoOperator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let action_str = self.as_str();
        write!(f, "{}", action_str)
    }
}

pub fn extract_operator(operator: &MongoOperator) -> &str {
    match operator {
        MongoOperator::EQ => "$eq",
        MongoOperator::NEQ => "$ne",
        MongoOperator::Exists => "$exists",
        MongoOperator::IN => "$in",
        MongoOperator::NIN => "$nin",
        MongoOperator::HasType => "$type",
        MongoOperator::ArrayContainsAll => "$all",
        MongoOperator::GT => "$gt",
        MongoOperator::GTE => "$gte",
        MongoOperator::LT => "$lt",
        MongoOperator::LTE => "$lte",
        MongoOperator::Regex => "$regex",
        MongoOperator::AND => "$and",
        MongoOperator::OR => "$or",
        MongoOperator::NOT => "$or",
        MongoOperator::NOR => "$nor",
    }
}
