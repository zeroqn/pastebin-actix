use actix_web::{pred, App};

use State;
use controllers::paste::*;

pub fn create(state: State) -> App<State> {
    App::with_state(state)
        .prefix("/pastes")
        .resource("/{id}", |r| {
            r.route().filter(pred::Get()).a(get_paste_by_id);
            r.route().filter(pred::Post()).a(update_paste_by_id);
            r.route().filter(pred::Delete()).a(del_paste_by_id);
        })
        .resource("", |r| {
            r.route().filter(pred::Post()).a(create_paste);
            r.route().filter(pred::Get()).with2(get_paste_list);
        })
}
