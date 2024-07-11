// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

use std::collections::HashMap;

pub trait Tree<K, V> {
    fn to_tree(&self) -> HashMap<K, V>;
}

// // Necesito traerlo aquí para poder implementarlo para estructuras de redis
// pub trait Show {
//     fn to_string(&self) -> String;
// }
