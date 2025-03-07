#![feature(
    unboxed_closures,
    impl_trait_in_fn_trait_return,
    tuple_trait,
    fn_traits
)]

pub mod tiktaktoe;

use std::{collections::HashMap, hash::Hash, marker::Tuple};

pub trait State {
    type Decision;
    fn decisions(&self) -> impl Iterator<Item = Self::Decision>;
    fn choose(&self, decision: Self::Decision) -> Self;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    Maximize,
    Minimize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Evaluation {
    Mode(Mode),
    ModeWithValue(Mode, f64),
    Value(f64),
}

pub trait Eval<State> {
    fn evaluate(&self, state: &State) -> Evaluation;
}

impl<S, T: Fn(&S) -> Evaluation> Eval<S> for T
where
    S: State,
{
    fn evaluate(&self, state: &S) -> Evaluation {
        self(state)
    }
}

/// cache takes a recursive function f and returns a new function that memoizes the results.
///
/// The function f is expected to accept as its first argument a recursive “call‐back”
/// that it can use to perform recursive calls (which in turn will be memoized),
/// and then the input value.
pub fn cache<I, F, O>(f: F) -> impl FnMut(I) -> O
where
    I: Clone + Hash + Eq,
    O: Clone,
    F: for<'a> Fn(&'a mut dyn FnMut(I) -> O, I) -> O,
{
    let mut cache = HashMap::new();

    fn rec<I, F, O>(i: I, f: &F, cache: &mut HashMap<I, O>) -> O
    where
        I: Clone + Hash + Eq,
        O: Clone,
        F: for<'a> Fn(&'a mut dyn FnMut(I) -> O, I) -> O,
    {
        if let Some(result) = cache.get(&i) {
            return result.clone();
        }

        let mut rec_closure = |i: I| rec(i, f, cache);
        let result = f(&mut rec_closure, i.clone());
        cache.insert(i, result.clone());
        result
    }

    move |i: I| rec(i, &f, &mut cache)
}

/// the ratio decides how much of the evaluation should be decided by minimax and how much by expected value
/// 0.0 means only expected value, 1.0 means only minimax
pub fn choose<S, E: Eval<S>>(eval: E, ratio: f64) -> impl FnMut(S) -> Option<(S::Decision, f64)>
where
    S: State + Clone + Eq + Hash,
    S::Decision: Clone + Eq + Hash,
{
    pub fn eval_helper<S: State>(
        state: S,
        mut eval: impl FnMut(S) -> f64,
        fold_value: f64,
        fold: impl Fn(f64, f64) -> f64,
        ratio: f64,
    ) -> f64 {
        let (minmax, expecto) = state
            .decisions()
            .map(|decision| eval(state.choose(decision)))
            .fold((fold_value, 0.0), |(f, sum), value| {
                (fold(f, value), sum + value)
            });
        ratio * minmax + (1.0 - ratio) * expecto
    }

    let mut evaluate = cache(move |evaluate, state: S| match eval.evaluate(&state) {
        Evaluation::Value(value) => value,
        Evaluation::Mode(Mode::Maximize) => {
            eval_helper(state, evaluate, f64::NEG_INFINITY, f64::max, ratio)
        }
        Evaluation::Mode(Mode::Minimize) => {
            eval_helper(state, evaluate, f64::INFINITY, f64::min, ratio)
        }
        Evaluation::ModeWithValue(Mode::Maximize, value) => {
            eval_helper(state, evaluate, f64::NEG_INFINITY, f64::max, ratio) + value
        }
        Evaluation::ModeWithValue(Mode::Minimize, value) => {
            eval_helper(state, evaluate, f64::INFINITY, f64::min, ratio) + value
        }
    });
    move |state| {
        state
            .decisions()
            .map(|decision| (decision.clone(), evaluate(state.choose(decision))))
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
    }
}
