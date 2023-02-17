use std::{io::{stdin, stdout, Write, self}, fmt::Display, num::NonZeroUsize, process::{Command, Stdio}, ffi::OsStr};

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
    println!("Done!");
    println!("{acons}");
    println!();
    println!("Trying to draw with dot!");
    acons.dot_draw("generated_tree.svg", "generated_tree.png").unwrap();
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
pub enum AnnotatedConstituent<'a, M> {
    APair(M, Box<AnnotatedConstituent<'a, M>>, Box<AnnotatedConstituent<'a, M>>),
    AWord(M, &'a str)
}

impl<M> AnnotatedConstituent<'_, M> {
    pub fn mark(&self) -> &M {
        match self {
            APair(m, _, _) => m,
            AWord(m, _) => m,
        }
    }
    fn dot_draw<P1: AsRef<OsStr>, P2: AsRef<OsStr>>(&self, path_svg: P1, path_png: P2) -> io::Result<()>
    where M: Display {
        let mut child = Command::new("dot")
            .arg("-Tsvg")
            .arg("-o")
            .arg(path_svg)
            .arg("-Tpng")
            .arg("-o")
            .arg(path_png)
            .arg("-Nshape=none")
            .arg("-Earrowhead=none")
            .stdin(Stdio::piped())
            .spawn()?;

        fn draw_node<'a, M: Display, W: Write>(w: &mut W, node: &AnnotatedConstituent<'a, M>, n: &mut impl Iterator<Item=usize>) -> io::Result<usize> {
            let n = match node {
                AWord(m, word) => {
                    let node_n = n.next().unwrap();
                    writeln!(w, "n{node_n} [fontcolor=blue label=\"{m}\"]")?;
                    let word_n = n.next().unwrap();
                    writeln!(w, "n{word_n} [label=\"{word}\"]")?;
                    writeln!(w, "n{node_n} -> n{word_n}")?;

                    node_n
                }
                APair(m, l, r) => {
                    let node_n = n.next().unwrap();
                    writeln!(w, "n{node_n} [fontcolor=blue label=\"{m}\"]")?;

                    let node_l = draw_node(w, l, n)?;
                    let node_r = draw_node(w, r, n)?;

                    writeln!(w, "n{node_n} -> {{n{node_l} n{node_r}}}")?;

                    node_n
                }
            };

            Ok(n)
        }

        {
            let i = child.stdin.as_mut().expect("stdin not piped");

            writeln!(i, "digraph {{")?;
            draw_node(i, &self, &mut (0..))?;
            writeln!(i, "}}")?;
        }

        child.wait()?;

        Ok(())
    }
}

impl<M: Display> Display for AnnotatedConstituent<'_, M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AWord(m, w) => write!(f, "[{m} {w}]"),
            APair(m, a, b) => write!(f, "[{m} {a} {b}]"),
        }
    }
}

fn ask_for_mark() -> String {
    print!("What is this? ");
    stdout().flush().unwrap();
    let mut answer = String::new();
    stdin().read_line(&mut answer).unwrap();

    answer.trim().to_owned()
}

fn interactively_annotate(constituent: Constituent) -> AnnotatedConstituent<String> {
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

// fn fix_constiuent(acons: &mut AnnotatedConstituent<Mark>) {
//     let mut phrase_queue = Vec::new();
//     fix_constiuent_helper(acons, &mut phrase_queue);
// }

// fn fix_constiuent_helper(acons: &mut AnnotatedConstituent<Mark>, phrase_queue: &mut Vec<char>) {
//     match acons {
//         AWord(mark, _) => {
//             match *mark {
//                 Phrase(c) | Bar(c) | Bare(c) => *mark = Bare(c),
//             };
//         }
//         APair(mark, l, r) => {
//             let lm = l.mark();
//             let rm = r.mark();

//             match *mark {
//                 Bare(c) => panic!("non-word constituent cannot be a part of speech"),
//                 Bar(c) => {
//                     todo!()
//                 }
//                 Phrase(c) => todo!(),
//             }
//         }
//     }
// }
