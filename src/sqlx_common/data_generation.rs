// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use chrono::DateTime;
use chrono::Utc;
use std::marker::PhantomData;

use crate::common::{
    generator::{Gen, SimpleRGen},
    traits::Runner,
};

// No aporta mucho tenerlo aquí, pero por no repetirlo mucho.
pub struct GenericGenerator<T> {
    _marker: PhantomData<T>,
}

impl Runner<DateTime<Utc>> for GenericGenerator<DateTime<Utc>> {
    fn run() -> DateTime<Utc> {
        Utc::now()
    }
}

impl Runner<i8> for GenericGenerator<i8> {
    fn run() -> i8 {
        let rng = SimpleRGen::new();
        Gen::gen_i8().sample(&rng)
    }
}

impl Runner<i16> for GenericGenerator<i16> {
    fn run() -> i16 {
        let rng = SimpleRGen::new();
        Gen::gen_i16().sample(&rng)
    }
}

impl Runner<i32> for GenericGenerator<i32> {
    fn run() -> i32 {
        let rng = SimpleRGen::new();
        Gen::gen_i32().sample(&rng)
    }
}

impl Runner<Vec<i32>> for GenericGenerator<Vec<i32>> {
    fn run() -> Vec<i32> {
        let rng = SimpleRGen::new();
        Gen::list_of(Gen::gen_i32()).sample(&rng)
    }
}

impl Runner<i64> for GenericGenerator<i64> {
    fn run() -> i64 {
        let rng = SimpleRGen::new();
        Gen::gen_i64().sample(&rng)
    }
}

impl Runner<Vec<i64>> for GenericGenerator<Vec<i64>> {
    fn run() -> Vec<i64> {
        let rng = SimpleRGen::new();
        Gen::list_of(Gen::gen_i64()).sample(&rng)
    }
}

impl Runner<u8> for GenericGenerator<u8> {
    fn run() -> u8 {
        let rng = SimpleRGen::new();
        let v = Gen::gen_i16().sample(&rng);

        if v < 0 {
            -v as u8
        } else {
            v as u8
        }
    }
}

impl Runner<u16> for GenericGenerator<u16> {
    fn run() -> u16 {
        let rng = SimpleRGen::new();
        let v = Gen::gen_i32().sample(&rng);

        if v < 0 {
            -v as u16
        } else {
            v as u16
        }
    }
}

impl Runner<u32> for GenericGenerator<u32> {
    fn run() -> u32 {
        let rng = SimpleRGen::new();
        let v = Gen::gen_i64().sample(&rng);

        if v < 0 {
            -v as u32
        } else {
            v as u32
        }
    }
}

impl Runner<u64> for GenericGenerator<u64> {
    fn run() -> u64 {
        let rng = SimpleRGen::new();
        Gen::gen_u64().sample(&rng)
    }
}

impl Runner<f32> for GenericGenerator<f32> {
    fn run() -> f32 {
        let rng = SimpleRGen::new();
        Gen::gen_f32().sample(&rng)
    }
}

impl Runner<f64> for GenericGenerator<f64> {
    fn run() -> f64 {
        let rng = SimpleRGen::new();
        Gen::gen_f64().sample(&rng)
    }
}

impl Runner<Vec<f64>> for GenericGenerator<Vec<f64>> {
    fn run() -> Vec<f64> {
        let rng = SimpleRGen::new();
        Gen::<f64, fn(&SimpleRGen) -> (f64, SimpleRGen)>::list_of_n(2, Gen::gen_f64()).sample(&rng)
    }
}

impl Runner<bool> for GenericGenerator<bool> {
    fn run() -> bool {
        let rng = SimpleRGen::new();
        Gen::gen_bool().sample(&rng)
    }
}
