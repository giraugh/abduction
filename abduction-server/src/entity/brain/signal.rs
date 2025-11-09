use rand::distr::{weighted::WeightedIndex, Distribution};

use crate::entity::{
    brain::{actor_action::ActorAction, focus::ActorFocus},
    snapshot::EntityView,
    world::EntityWorld,
    Entity,
};

/// Information available when resolving a signal into actions
/// (Extends the action context)
#[derive(Debug)]
pub struct SignalContext<'a> {
    /// A view of all entities from the snapshot
    pub entities: &'a EntityView<'a>,

    /// The entity having its signal resolved
    pub entity: &'a Entity,

    /// The current focus of the entity having its signal resolved
    pub focus: ActorFocus,

    /// The current world state
    pub world_state: &'a EntityWorld,
}

/// Something that a player acts on -> can raise weighted actions
pub trait Signal: std::fmt::Debug {
    fn act_on(&self, ctx: &SignalContext, actions: &mut WeightedActorActions);
}

/// Helper for a dynamic signal object. This allows for wrapping and owning where required (`Boxed`) or
/// just passing a reference when possible (`Ref`)
#[derive(Debug)]
pub enum SignalRef<'a> {
    Boxed(Box<dyn Signal>),
    Ref(&'a dyn Signal),
}

impl<'a> SignalRef<'a> {
    pub fn boxed<T: Signal + 'static>(value: T) -> Self {
        Self::Boxed(Box::new(value) as Box<dyn Signal>)
    }

    pub fn reference<T: Signal>(value: &'a T) -> Self {
        Self::Ref(value)
    }
}

impl Signal for SignalRef<'_> {
    fn act_on(&self, ctx: &SignalContext, actions: &mut WeightedActorActions) {
        match self {
            SignalRef::Boxed(signal) => signal.act_on(ctx, actions),
            SignalRef::Ref(signal) => signal.act_on(ctx, actions),
        }
    }
}

/// Actions and their weights as returned by a signal implementor
#[derive(Debug, Clone, Default)]
pub struct WeightedActorActions {
    actions: Option<Vec<(usize, ActorAction)>>,
}

impl WeightedActorActions {
    pub fn sample(mut self, rng: &mut impl rand::Rng) -> ActorAction {
        // Add no-op if no actions
        if self.actions.is_none() {
            self.add(1, ActorAction::Nothing);
        }

        // Build the distribution
        let (weights, actions): (Vec<_>, Vec<_>) = self.actions.unwrap().into_iter().unzip();
        let dist = WeightedIndex::new(&weights).unwrap();

        // Sample the distribution
        actions[dist.sample(rng)].clone()
    }
}

impl WeightedActorActions {
    pub fn add(&mut self, weight: usize, action: ActorAction) {
        self.actions.get_or_insert_default().push((weight, action));
    }

    pub fn extend(&mut self, actions: impl Iterator<Item = (usize, ActorAction)>) {
        self.actions.get_or_insert_default().extend(actions);
    }
}
