// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

// Distintas macros para uso de funciones muy repetitivas o para incluir
// información extra.
#[macro_export]
macro_rules! qk_info {
    ($($arg:tt)*) => ({
        let now = chrono::Local::now();
        println!("[INFO]   {}: {}", now.to_rfc3339(), format_args!($($arg)*));
    })
}

#[macro_export]
macro_rules! qk_error {
    ($($arg:tt)*) => ({
        let now = chrono::Local::now();
        eprintln!("[ERROR] {}: {}", now.to_rfc3339(), format_args!($($arg)*));
    })
}

// Versión más potente, pero quiero algo muy rápido así que opto por la implementación sencilla.
// #[macro_export]
// macro_rules! quote {
//     ($($arg:tt)*) => ({
//         format!("'{}'", format_args!($($arg)*))
//     });
// }

#[macro_export]
macro_rules! quote {
    ($s:expr) => {
        format!("'{}'", $s)
    };
}
