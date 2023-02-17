use core::panic;
use std::{io::{stdin, stdout, Write}, fmt::Display, num::NonZeroUsize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Constituent<'a> {
    Pair(Box<Constituent<'a>>, Box<Constituent<'a>>),
    Word(&'a str)
}

impl Display for Constituent<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Word(w) => w.fmt(f),
            Pair(a, b) => write!(f, "[{a}] [{b}]"),
        }
    }
}

use self::Constituent::*;

fn main() {
    println!("Sentence: ");
    let mut sentence = String::new();
    stdin().read_line(&mut sentence).unwrap();
    let mut consituent_list = Vec::new();

    for word in sentence.trim().split_whitespace() {
        consituent_list.push(Word(word));
    }

    while consituent_list.len() > 1 {
        println!("Constituents: ");
        for (i, constituent) in consituent_list.iter().enumerate() {
            println!(" {}. {constituent}", i+1);
        }
        print!("Which two should merge? ");
        stdout().flush().unwrap();

        let Some((i, j)) = get_indices() else {
            eprintln!("You need to write two numbers from the constituency list");
            continue;
        };

        if j.get() - i.get() != 1 {
            eprintln!("The constiuents must be adjacent!");
            continue;
        }

        let second = consituent_list.remove(j.get() - 1);
        let first = consituent_list.get_mut(i.get() - 1).unwrap();

        *first = Pair(Box::new(first.clone()), Box::new(second));
    }

    println!("Done!");
    let cons = consituent_list.remove(0);
    println!("[{cons}]");

    println!();
    println!("Now let's annotate it");
    let acons = interactively_annotate(cons);
    println!("Put this into a syntax tree drawer:");
    println!("{acons}");
    println!();
}

fn get_indices() -> Option<(NonZeroUsize, NonZeroUsize)> {
    let mut answer = String::new();
    stdin().read_line(&mut answer).unwrap();
    let mut indices = answer.trim().split(|c: char| !c.is_numeric()).filter(|p| !p.is_empty());
    let i: NonZeroUsize = indices.next()?.parse().ok()?;
    let j: NonZeroUsize = indices.next()?.parse().ok()?;

    if indices.next().is_some() { return None; }

    Some((i.min(j), i.max(j)))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mark {
    Phrase(char),
    Bar(char),
    Bare(char),
}
use Mark::*;
use AnnotatedConstituent::*;

impl Display for Mark {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Phrase(c) => write!(f, "{c}P"),
            Bar(c) => write!(f, "{c}'"),
            Bare(c) => write!(f, "{c}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnnotatedConstituent<'a> {
    APair(Mark, Box<AnnotatedConstituent<'a>>, Box<AnnotatedConstituent<'a>>),
    AWord(Mark, &'a str)
}

impl AnnotatedConstituent<'_> {
    fn mark(&self) -> Mark {
        match *self {
            APair(m, _, _) => m,
            AWord(m, _) => m,
        }
    }
}

impl Display for AnnotatedConstituent<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AWord(m, w) => write!(f, "[{m} {w}]"),
            APair(m, a, b) => write!(f, "[{m} {a} {b}]"),
        }
    }
}

fn ask_for_mark() -> Mark {
    print!("What is this? ");
    stdout().flush().unwrap();
    let mut answer = String::new();
    stdin().read_line(&mut answer).unwrap();

    let mut chars = answer.trim().chars();
    let pos = chars.next().unwrap();
    let p_type = chars.next();
    assert!(chars.next().is_none());

    match p_type {
        Some('P') => Phrase(pos),
        Some('\'') => Bar(pos),
        None => Bare(pos),
        _ => panic!("unknown type"),
    }
}

fn interactively_annotate(constituent: Constituent) -> AnnotatedConstituent {
    match constituent {
        Word(word) => {
            println!("Constiuent [? {word}]");
            let mark = ask_for_mark();
            AWord(mark, word)
        }
        Pair(left, right) => {
            let left = interactively_annotate(*left);
            let right = interactively_annotate(*right);
            println!("Constiuent [? {left} {right}]");
            let mark = ask_for_mark();
            APair(mark, Box::new(left), Box::new(right))
        }
    }
}

fn fix_constiuent(acons: &mut AnnotatedConstituent) {
    let mut phrase_queue = Vec::new();
    fix_constiuent_helper(acons, &mut phrase_queue);
}

fn fix_constiuent_helper(acons: &mut AnnotatedConstituent, phrase_queue: &mut Vec<char>) {
    match acons {
        AWord(mark, _) => {
            match *mark {
                Phrase(c) | Bar(c) | Bare(c) => *mark = Bare(c),
            };
        }
        APair(mark, l, r) => {
            let lm = l.mark();
            let rm = r.mark();

            match *mark {
                Bare(c) => panic!("non-word constituent cannot be a part of speech"),
                Bar(c) => {
                    todo!()
                }
                Phrase(c) => todo!(),
            }
        }
    }
}