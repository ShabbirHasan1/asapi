// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------

/// Módulo con elementos comunes a todos los módulos.
///
/// Traits, structs, funciones... cualquier elemento que sea común a varios
/// módulos. Para aquellos que sean comunes a varios módulos dependientes de
/// sqlx, allí están los elementos comunes. Si hay otros a parte de sqlx,
/// vienen aquí.
pub mod fs;
pub mod generator;
pub mod internationalization;
pub mod macros;
pub mod syntax_highlighting;
pub mod traits;
