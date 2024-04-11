// -------------------------------------------------------------------------
// Copyright (C) 2024 Fernando López Laso - All Rights Reserved
//
// Unauthorized copying of this file, via any medium is strictly prohibited.
// This file is confidential and only available to authorized individuals
// with the permission of the copyright holders.
// -------------------------------------------------------------------------
use crate::{
    components::toggle_selector::toggle_label,
    mongom::{actions::MongoAction, bson_type::BsonType, filter::MongoOperator, view::MongoView},
};
use eframe::egui;

/// Para la gestión de como dice el nombre del archivo los combo boxes.
///
/// Lo extraigo porque son elementos tan largos que me merece la pena
/// tener un archivo externo para ellos solo para no molestar en la navegación
/// en el archivo padre.
/// Los creo como implementación de la vista de mongo para simplificar.

impl MongoView {
    pub fn available_keys_combo(&mut self, ui: &mut egui::Ui) {
        egui::ComboBox::from_id_source("selected_col_available_keys")
            .selected_text(self.state.current_selected_key.as_str())
            .show_ui(ui, |ui| {
                self.state.current_available_keys.iter().for_each(|k| {
                    if ui
                        .selectable_value(&mut self.state.current_selected_key, k.to_owned(), k)
                        .clicked()
                    {
                        // Tengo unn listado de documentos, en el que puede o no estar la clave
                        // seleccionada. Tiene que estar por cómo se genera el `HashSet`, así
                        // que hago esto, pero podría ser muy arriesgado.
                        let mut idx = 0;
                        let len = self.state.current_col_find_document_result.len();
                        loop {
                            if idx == len {
                                break;
                            }
                            let opt_bson = self.state.current_col_find_document_result[idx].get(k);
                            if let Some(bs) = opt_bson {
                                self.state.current_selected_type_bson_type = BsonType::from(bs);
                                break;
                            }
                            idx += 1;
                        }
                    }
                });
            });
    }

    pub fn action_selector_header(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.add_enabled_ui(
                self.state.selected_action == MongoAction::Find
                    || self.state.selected_action == MongoAction::FindOne,
                |ui| {
                    toggle_label(
                        ui,
                        &mut self.state.current_selection.show_user_free_input,
                        "User Defined",
                    );
                },
            );

            ui.separator();

            ui.selectable_value(&mut self.state.selected_action, MongoAction::Find, "Find");

            ui.selectable_value(
                &mut self.state.selected_action,
                MongoAction::FindOne,
                "Find One",
            );

            ui.selectable_value(
                &mut self.state.selected_action,
                MongoAction::InsertMany,
                "Insert Many",
            );

            ui.selectable_value(
                &mut self.state.selected_action,
                MongoAction::InsertOne,
                "Insert One",
            );

            ui.selectable_value(
                &mut self.state.selected_action,
                MongoAction::UpdateMany,
                "Update Many",
            );

            ui.selectable_value(
                &mut self.state.selected_action,
                MongoAction::UpdateOne,
                "Update One",
            );

            ui.selectable_value(
                &mut self.state.selected_action,
                MongoAction::DeleteMany,
                "Delete Many",
            );

            ui.selectable_value(
                &mut self.state.selected_action,
                MongoAction::DeleteOne,
                "Delete One",
            );

            ui.selectable_value(
                &mut self.state.selected_action,
                MongoAction::ReplaceMany,
                "Replace Many",
            );

            ui.selectable_value(
                &mut self.state.selected_action,
                MongoAction::ReplaceOne,
                "Replace One",
            );
        });
    }

    // Podría iterar sobre MongoFindfilter::varaints, pero al final
    // al iterar obtengo un `&MongoFindFilter::variante` y me toca
    // clonar para pasar al `selectable_value`, por lo que no ahorro.
    pub fn select_action_options(&mut self, ui: &mut egui::Ui) {
        egui::ComboBox::from_id_source("MongoAction")
            .selected_text(self.state.current_operator.as_str())
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut self.state.current_operator,
                    MongoOperator::EQ,
                    MongoOperator::EQ.as_str(),
                );
                ui.selectable_value(
                    &mut self.state.current_operator,
                    MongoOperator::NEQ,
                    MongoOperator::NEQ.as_str(),
                );
                ui.selectable_value(
                    &mut self.state.current_operator,
                    MongoOperator::IN,
                    MongoOperator::IN.as_str(),
                );
                ui.selectable_value(
                    &mut self.state.current_operator,
                    MongoOperator::NIN,
                    MongoOperator::NIN.as_str(),
                );
                ui.selectable_value(
                    &mut self.state.current_operator,
                    MongoOperator::HasType,
                    MongoOperator::HasType.as_str(),
                );
                ui.selectable_value(
                    &mut self.state.current_operator,
                    MongoOperator::ArrayContainsAll,
                    MongoOperator::ArrayContainsAll.as_str(),
                );
                ui.selectable_value(
                    &mut self.state.current_operator,
                    MongoOperator::GT,
                    MongoOperator::GT.as_str(),
                );
                ui.selectable_value(
                    &mut self.state.current_operator,
                    MongoOperator::GTE,
                    MongoOperator::GTE.as_str(),
                );
                ui.selectable_value(
                    &mut self.state.current_operator,
                    MongoOperator::LT,
                    MongoOperator::LT.as_str(),
                );
                ui.selectable_value(
                    &mut self.state.current_operator,
                    MongoOperator::LTE,
                    MongoOperator::LTE.as_str(),
                );
                ui.selectable_value(
                    &mut self.state.current_operator,
                    MongoOperator::Regex,
                    MongoOperator::Regex.as_str(),
                );
            });
    }

    pub fn select_bson_data_type(&mut self, ui: &mut egui::Ui) {
        egui::ComboBox::from_id_source("MongoBsonDateType")
            .selected_text(self.state.current_selected_type_bson_type.as_str())
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut self.state.current_selected_type_bson_type,
                    BsonType::Double,
                    BsonType::Double.as_str(),
                );
                ui.selectable_value(
                    &mut self.state.current_selected_type_bson_type,
                    BsonType::String,
                    BsonType::String.as_str(),
                );
                ui.selectable_value(
                    &mut self.state.current_selected_type_bson_type,
                    BsonType::Array,
                    BsonType::Array.as_str(),
                );
                ui.selectable_value(
                    &mut self.state.current_selected_type_bson_type,
                    BsonType::Document,
                    BsonType::Document.as_str(),
                );
                ui.selectable_value(
                    &mut self.state.current_selected_type_bson_type,
                    BsonType::Boolean,
                    BsonType::Boolean.as_str(),
                );
                ui.selectable_value(
                    &mut self.state.current_selected_type_bson_type,
                    BsonType::Null,
                    BsonType::Null.as_str(),
                );
                ui.selectable_value(
                    &mut self.state.current_selected_type_bson_type,
                    BsonType::RegularExpression,
                    BsonType::RegularExpression.as_str(),
                );
                ui.selectable_value(
                    &mut self.state.current_selected_type_bson_type,
                    BsonType::Int32,
                    BsonType::Int32.as_str(),
                );
                ui.selectable_value(
                    &mut self.state.current_selected_type_bson_type,
                    BsonType::Int64,
                    BsonType::Int64.as_str(),
                );
                ui.selectable_value(
                    &mut self.state.current_selected_type_bson_type,
                    BsonType::Timestamp,
                    BsonType::Timestamp.as_str(),
                );
                ui.selectable_value(
                    &mut self.state.current_selected_type_bson_type,
                    BsonType::Binary,
                    BsonType::Binary.as_str(),
                );
                ui.selectable_value(
                    &mut self.state.current_selected_type_bson_type,
                    BsonType::ObjectId,
                    BsonType::ObjectId.as_str(),
                );
                ui.selectable_value(
                    &mut self.state.current_selected_type_bson_type,
                    BsonType::DateTime,
                    BsonType::DateTime.as_str(),
                );
                ui.selectable_value(
                    &mut self.state.current_selected_type_bson_type,
                    BsonType::Symbol,
                    BsonType::Symbol.as_str(),
                );
                ui.selectable_value(
                    &mut self.state.current_selected_type_bson_type,
                    BsonType::Decimal128,
                    BsonType::Decimal128.as_str(),
                );
            });
    }
}
