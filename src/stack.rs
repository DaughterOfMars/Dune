use bevy::prelude::*;

use crate::components::CardEffect;

pub struct EffectStack(pub Vec<CardEffect>);

impl Default for EffectStack {
    fn default() -> Self {
        EffectStack(Vec::new())
    }
}

impl EffectStack {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn push(&mut self, action: CardEffect) {
        self.0.push(action);
    }

    pub fn peek(&self) -> Option<&CardEffect> {
        self.0.last()
    }

    pub fn peek_mut(&mut self) -> Option<&mut CardEffect> {
        self.0.last_mut()
    }

    pub fn pop(&mut self) -> Option<CardEffect> {
        self.0.pop()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn extend<T: IntoIterator<Item = CardEffect>>(&mut self, iter: T) {
        self.0.extend(iter);
    }
}

pub fn effects_system(mut stack: ResMut<EffectStack>) {
    if let Some(action) = stack.peek_mut() {
        match action {
            CardEffect::Worthless => {}
            CardEffect::PoisonWeapon => {}
            CardEffect::ProjectileWeapon => {}
            CardEffect::CheapHero => {}
            CardEffect::PoisonDefense => {}
            CardEffect::ProjectileDefense => {}
            CardEffect::Atomics => {}
            CardEffect::Movement => {}
            CardEffect::Karama => {}
            CardEffect::Lasgun => {}
            CardEffect::Revive => {}
            CardEffect::Truthtrance => {}
            CardEffect::WeatherControl => {}
        }
    }
}
