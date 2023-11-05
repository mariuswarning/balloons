mod geometry;
mod logic;
mod quadtree;
mod rendering;
mod utils;
mod constants;
mod ball;
mod random;


use logic::GameState;

use js_sys::Function;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{KeyboardEvent, MouseEvent};

use crate::utils::{get_context, request_animation_frame, window};

thread_local! {
      static GAME: Rc<RefCell<GameState>> = Rc::new(
        RefCell::new(
            GameState::new(
                constants::WIDTH,
                constants::HEIGHT))
    );

    static HANDLE_KEYDOWN: Closure<dyn FnMut(KeyboardEvent)> =
    Closure::wrap(Box::new(|evt: KeyboardEvent| GAME.with(
        |game| {
            game.borrow_mut().handle_keydown_event(&evt.key()[..]);
        }
    )) as Box<dyn FnMut(KeyboardEvent)>);

    static HANDLE_MOUSE: Closure<dyn FnMut(MouseEvent)> =
    Closure::wrap(Box::new(|evt: MouseEvent| GAME.with(
        |game| {
            game.borrow_mut().handle_mouse_event(evt.button(), evt.offset_x() as f64, evt.offset_y() as f64);
        }
    )) as Box<dyn FnMut(MouseEvent)>);
}

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue> {
    let f = Rc::new(RefCell::new(None));
    let g = f.clone();
    let mut ctx = get_context();
    HANDLE_KEYDOWN.with(|handle_keydown| {
        window()
            .add_event_listener_with_callback(
                "keydown",
                handle_keydown.as_ref().dyn_ref::<Function>().unwrap_throw(),
            )
            .unwrap_throw();
    });

    HANDLE_MOUSE.with(|handle_mousedown| {
        window()
            .add_event_listener_with_callback(
                "mousedown",
                handle_mousedown
                    .as_ref()
                    .dyn_ref::<Function>()
                    .unwrap_throw(),
            )
            .unwrap_throw();
    });
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        GAME.with(|game| {
            game.borrow_mut().tick();
            game.borrow().render_state(&mut ctx, true);
        });

        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    request_animation_frame(g.borrow().as_ref().unwrap());
    Ok(())
}
