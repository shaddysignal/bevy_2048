use bevy::prelude::*;

#[derive(States, Copy, Clone, Debug, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    Win,
    Lose,
    #[default]
    Wait,
    Process,
    Movement,
    Decision,
}

#[derive(SystemSet, Clone, Eq, PartialEq, Hash, Debug)]
pub struct GameSet;

#[derive(SystemSet, Clone, Eq, PartialEq, Hash, Debug)]
pub struct InputSet;

// TODO: create separate system sets for different types of systems
// #[derive(SystemSet, Clone, Eq, PartialEq, Hash, Debug)]
// pub struct EffectSet;
//
// #[derive(SystemSet, Clone, Eq, PartialEq, Hash, Debug)]
// pub struct AudioSet;

// TODO: Keep only when not release?
pub fn log_transitions<S: States>(mut transitions: MessageReader<StateTransitionEvent<S>>) {
    // State internals can generate at most one event (of type) per frame.
    let Some(transition) = transitions.read().last() else {
        return;
    };
    let name = core::any::type_name::<S>();
    let StateTransitionEvent { exited, entered } = transition;
    trace!("{} transition: {:?} => {:?}", name, exited, entered);
}