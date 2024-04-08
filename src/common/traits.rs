// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

pub trait ShowVec {
    fn to_string_vec(&self) -> Vec<String>;
}

pub trait Show {
    fn to_string(&self) -> String;
}

pub trait Runner<T> {
    fn run() -> T;
}
