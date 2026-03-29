#[derive(Debug, Clone)]
pub struct LanguageSample {
    pub language_id: String,
    pub file_extension: String,
    pub content: String,
    pub hover_line: i64,
    pub hover_char: i64,
    pub signature_line: i64,
    pub signature_char: i64,
    pub completion_line: i64,
    pub completion_char: i64,
    pub definition_line: i64,
    pub definition_char: i64,
    pub references_line: i64,
    pub references_char: i64,
    pub rename_line: i64,
    pub rename_char: i64,
}

impl LanguageSample {
    pub fn uri(&self) -> String {
        if cfg!(windows) {
            format!("file:///C:/Temp/lsp_health_check{}", self.file_extension)
        } else {
            format!("file:///tmp/lsp_health_check{}", self.file_extension)
        }
    }
}

pub fn get_sample(language: &str) -> Option<LanguageSample> {
    match language.to_lowercase().as_str() {
        "rust" => Some(LanguageSample {
            language_id: "rust".to_string(),
            file_extension: ".rs".to_string(),
            content: r#"fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn main() {
    let x = add(1, 2);
    let _ = x;
}
"#
            .to_string(),
            hover_line: 0,
            hover_char: 7,
            signature_line: 5,
            signature_char: 19,
            completion_line: 5,
            completion_char: 14,
            definition_line: 5,
            definition_char: 18,
            references_line: 0,
            references_char: 7,
            rename_line: 0,
            rename_char: 7,
        }),
        "c" => Some(LanguageSample {
            language_id: "c".to_string(),
            file_extension: ".c".to_string(),
            content: r#"int add(int a, int b) {
    return a + b;
}

int main() {
    int x = add(1, 2);
    return 0;
}
"#
            .to_string(),
            hover_line: 0,
            hover_char: 5,
            signature_line: 5,
            signature_char: 17,
            completion_line: 5,
            completion_char: 12,
            definition_line: 5,
            definition_char: 16,
            references_line: 0,
            references_char: 5,
            rename_line: 0,
            rename_char: 5,
        }),
        "cpp" | "c++" => Some(LanguageSample {
            language_id: "cpp".to_string(),
            file_extension: ".cpp".to_string(),
            content: r#"int add(int a, int b) {
    return a + b;
}

int main() {
    int x = add(1, 2);
    return 0;
}
"#
            .to_string(),
            hover_line: 0,
            hover_char: 5,
            signature_line: 5,
            signature_char: 17,
            completion_line: 5,
            completion_char: 12,
            definition_line: 5,
            definition_char: 16,
            references_line: 0,
            references_char: 5,
            rename_line: 0,
            rename_char: 5,
        }),
        "python" | "py" => Some(LanguageSample {
            language_id: "python".to_string(),
            file_extension: ".py".to_string(),
            content: r#"def add(a: int, b: int) -> int:
    return a + b

def main():
    x = add(1, 2)
    return x
"#
            .to_string(),
            hover_line: 0,
            hover_char: 4,
            signature_line: 4,
            signature_char: 12,
            completion_line: 4,
            completion_char: 8,
            definition_line: 4,
            definition_char: 11,
            references_line: 0,
            references_char: 4,
            rename_line: 0,
            rename_char: 4,
        }),
        "d" => Some(LanguageSample {
            language_id: "d".to_string(),
            file_extension: ".d".to_string(),
            content: r#"int add(int a, int b) {
    return a + b;
}

int main() {
    int x = add(1, 2);
    return 0;
}
"#
            .to_string(),
            hover_line: 0,
            hover_char: 5,
            signature_line: 5,
            signature_char: 17,
            completion_line: 5,
            completion_char: 12,
            definition_line: 5,
            definition_char: 16,
            references_line: 0,
            references_char: 5,
            rename_line: 0,
            rename_char: 5,
        }),
        "zig" => Some(LanguageSample {
            language_id: "zig".to_string(),
            file_extension: ".zig".to_string(),
            content: r#"const std = @import("std");

fn add(a: i32, b: i32) i32 {
    return a + b;
}

pub fn main() !void {
    const x = add(1, 2);
    _ = x;
}
"#
            .to_string(),
            hover_line: 2,
            hover_char: 7,
            signature_line: 7,
            signature_char: 19,
            completion_line: 7,
            completion_char: 14,
            definition_line: 7,
            definition_char: 18,
            references_line: 2,
            references_char: 7,
            rename_line: 2,
            rename_char: 7,
        }),
        "csharp" | "cs" => Some(LanguageSample {
            language_id: "csharp".to_string(),
            file_extension: ".cs".to_string(),
            content: r#"class Program {
    static int Add(int a, int b) {
        return a + b;
    }

    static void Main() {
        int x = Add(1, 2);
    }
}
"#
            .to_string(),
            hover_line: 1,
            hover_char: 17,
            signature_line: 6,
            signature_char: 20,
            completion_line: 6,
            completion_char: 16,
            definition_line: 6,
            definition_char: 19,
            references_line: 1,
            references_char: 17,
            rename_line: 1,
            rename_char: 17,
        }),
        "nim" => Some(LanguageSample {
            language_id: "nim".to_string(),
            file_extension: ".nim".to_string(),
            content: r#"func add(a: int, b: int): int =
    a + b

func main() =
    let x = add(1, 2)
    discard x
"#
            .to_string(),
            hover_line: 0,
            hover_char: 5,
            signature_line: 4,
            signature_char: 14,
            completion_line: 4,
            completion_char: 12,
            definition_line: 4,
            definition_char: 13,
            references_line: 0,
            references_char: 5,
            rename_line: 0,
            rename_char: 5,
        }),
        "hare" => Some(LanguageSample {
            language_id: "hare".to_string(),
            file_extension: ".ha".to_string(),
            content: r#"fn add(a: int, b: int) int = {
    return a + b;
};

fn main() void = {
    let x = add(1, 2);
};
"#
            .to_string(),
            hover_line: 0,
            hover_char: 7,
            signature_line: 5,
            signature_char: 16,
            completion_line: 5,
            completion_char: 12,
            definition_line: 5,
            definition_char: 15,
            references_line: 0,
            references_char: 7,
            rename_line: 0,
            rename_char: 7,
        }),
        "scheme" | "scm" => Some(LanguageSample {
            language_id: "scheme".to_string(),
            file_extension: ".scm".to_string(),
            content: r#"(define (add a b)
  (+ a b))

(define (main)
  (let ((x (add 1 2)))
    x))
"#
            .to_string(),
            hover_line: 0,
            hover_char: 8,
            signature_line: 4,
            signature_char: 15,
            completion_line: 4,
            completion_char: 12,
            definition_line: 4,
            definition_char: 14,
            references_line: 0,
            references_char: 8,
            rename_line: 0,
            rename_char: 8,
        }),
        "java" => Some(LanguageSample {
            language_id: "java".to_string(),
            file_extension: ".java".to_string(),
            content: r#"class Main {
    static int add(int a, int b) {
        return a + b;
    }

    public static void main(String[] args) {
        int x = add(1, 2);
    }
}
"#
            .to_string(),
            hover_line: 1,
            hover_char: 17,
            signature_line: 6,
            signature_char: 20,
            completion_line: 6,
            completion_char: 16,
            definition_line: 6,
            definition_char: 19,
            references_line: 1,
            references_char: 17,
            rename_line: 1,
            rename_char: 17,
        }),
        "kotlin" | "kt" => Some(LanguageSample {
            language_id: "kotlin".to_string(),
            file_extension: ".kt".to_string(),
            content: r#"fun add(a: Int, b: Int): Int {
    return a + b
}

fun main() {
    val x = add(1, 2)
}
"#
            .to_string(),
            hover_line: 0,
            hover_char: 4,
            signature_line: 5,
            signature_char: 14,
            completion_line: 5,
            completion_char: 12,
            definition_line: 5,
            definition_char: 13,
            references_line: 0,
            references_char: 4,
            rename_line: 0,
            rename_char: 4,
        }),
        "crystal" | "cr" => Some(LanguageSample {
            language_id: "crystal".to_string(),
            file_extension: ".cr".to_string(),
            content: r#"def add(a : Int32, b : Int32) : Int32
  a + b
end

def main
  x = add(1, 2)
end
"#
            .to_string(),
            hover_line: 0,
            hover_char: 4,
            signature_line: 5,
            signature_char: 8,
            completion_line: 5,
            completion_char: 6,
            definition_line: 5,
            definition_char: 7,
            references_line: 0,
            references_char: 4,
            rename_line: 0,
            rename_char: 4,
        }),
        "perl" | "pl" => Some(LanguageSample {
            language_id: "perl".to_string(),
            file_extension: ".pl".to_string(),
            content: r#"sub add {
    my ($a, $b) = @_;
    return $a + $b;
}

sub main {
    my $x = add(1, 2);
    return $x;
}
"#
            .to_string(),
            hover_line: 0,
            hover_char: 4,
            signature_line: 6,
            signature_char: 16,
            completion_line: 6,
            completion_char: 12,
            definition_line: 6,
            definition_char: 15,
            references_line: 0,
            references_char: 4,
            rename_line: 0,
            rename_char: 4,
        }),
        "javascript" | "js" => Some(LanguageSample {
            language_id: "javascript".to_string(),
            file_extension: ".js".to_string(),
            content: r#"function add(a, b) {
    return a + b;
}

function main() {
    const x = add(1, 2);
    return x;
}
"#
            .to_string(),
            hover_line: 0,
            hover_char: 9,
            signature_line: 5,
            signature_char: 19,
            completion_line: 5,
            completion_char: 14,
            definition_line: 5,
            definition_char: 18,
            references_line: 0,
            references_char: 9,
            rename_line: 0,
            rename_char: 9,
        }),
        "typescript" | "ts" => Some(LanguageSample {
            language_id: "typescript".to_string(),
            file_extension: ".ts".to_string(),
            content: r#"function add(a: number, b: number): number {
    return a + b;
}

function main(): number {
    const x = add(1, 2);
    return x;
}
"#
            .to_string(),
            hover_line: 0,
            hover_char: 9,
            signature_line: 5,
            signature_char: 19,
            completion_line: 5,
            completion_char: 14,
            definition_line: 5,
            definition_char: 18,
            references_line: 0,
            references_char: 9,
            rename_line: 0,
            rename_char: 9,
        }),
        "go" => Some(LanguageSample {
            language_id: "go".to_string(),
            file_extension: ".go".to_string(),
            content: r#"package main

func add(a int, b int) int {
    return a + b
}

func main() {
    x := add(1, 2)
    _ = x
}
"#
            .to_string(),
            hover_line: 2,
            hover_char: 5,
            signature_line: 7,
            signature_char: 11,
            completion_line: 7,
            completion_char: 9,
            definition_line: 7,
            definition_char: 10,
            references_line: 2,
            references_char: 5,
            rename_line: 2,
            rename_char: 5,
        }),
        "ruby" | "rb" => Some(LanguageSample {
            language_id: "ruby".to_string(),
            file_extension: ".rb".to_string(),
            content: r#"def add(a, b)
  a + b
end

def main
  x = add(1, 2)
  x
end
"#
            .to_string(),
            hover_line: 0,
            hover_char: 4,
            signature_line: 5,
            signature_char: 9,
            completion_line: 5,
            completion_char: 6,
            definition_line: 5,
            definition_char: 8,
            references_line: 0,
            references_char: 4,
            rename_line: 0,
            rename_char: 4,
        }),
        "php" => Some(LanguageSample {
            language_id: "php".to_string(),
            file_extension: ".php".to_string(),
            content: r#"<?php
function add($a, $b) {
    return $a + $b;
}

function main() {
    $x = add(1, 2);
    return $x;
}
"#
            .to_string(),
            hover_line: 1,
            hover_char: 9,
            signature_line: 6,
            signature_char: 14,
            completion_line: 6,
            completion_char: 9,
            definition_line: 6,
            definition_char: 13,
            references_line: 1,
            references_char: 9,
            rename_line: 1,
            rename_char: 9,
        }),
        "swift" => Some(LanguageSample {
            language_id: "swift".to_string(),
            file_extension: ".swift".to_string(),
            content: r#"func add(a: Int, b: Int) -> Int {
    return a + b
}

func main() {
    let x = add(a: 1, b: 2)
}
"#
            .to_string(),
            hover_line: 0,
            hover_char: 5,
            signature_line: 5,
            signature_char: 16,
            completion_line: 5,
            completion_char: 12,
            definition_line: 5,
            definition_char: 15,
            references_line: 0,
            references_char: 5,
            rename_line: 0,
            rename_char: 5,
        }),
        "lua" => Some(LanguageSample {
            language_id: "lua".to_string(),
            file_extension: ".lua".to_string(),
            content: r#"function add(a, b)
    return a + b
end

function main()
    local x = add(1, 2)
    return x
end
"#
            .to_string(),
            hover_line: 0,
            hover_char: 9,
            signature_line: 5,
            signature_char: 18,
            completion_line: 5,
            completion_char: 14,
            definition_line: 5,
            definition_char: 17,
            references_line: 0,
            references_char: 9,
            rename_line: 0,
            rename_char: 9,
        }),
        "r" => Some(LanguageSample {
            language_id: "r".to_string(),
            file_extension: ".r".to_string(),
            content: r#"add <- function(a, b) {
  a + b
}

main <- function() {
  x <- add(1, 2)
  x
}
"#
            .to_string(),
            hover_line: 0,
            hover_char: 0,
            signature_line: 5,
            signature_char: 9,
            completion_line: 5,
            completion_char: 7,
            definition_line: 5,
            definition_char: 8,
            references_line: 0,
            references_char: 0,
            rename_line: 0,
            rename_char: 0,
        }),
        "haskell" | "hs" => Some(LanguageSample {
            language_id: "haskell".to_string(),
            file_extension: ".hs".to_string(),
            content: r#"add :: Int -> Int -> Int
add a b = a + b

main :: IO ()
main = do
  let x = add 1 2
  return ()
"#
            .to_string(),
            hover_line: 1,
            hover_char: 0,
            signature_line: 5,
            signature_char: 12,
            completion_line: 5,
            completion_char: 10,
            definition_line: 5,
            definition_char: 11,
            references_line: 1,
            references_char: 0,
            rename_line: 1,
            rename_char: 0,
        }),
        "elixir" | "ex" | "exs" => Some(LanguageSample {
            language_id: "elixir".to_string(),
            file_extension: ".ex".to_string(),
            content: r#"defmodule Main do
  def add(a, b) do
    a + b
  end

  def main do
    x = add(1, 2)
    x
  end
end
"#
            .to_string(),
            hover_line: 1,
            hover_char: 6,
            signature_line: 6,
            signature_char: 10,
            completion_line: 6,
            completion_char: 8,
            definition_line: 6,
            definition_char: 9,
            references_line: 1,
            references_char: 6,
            rename_line: 1,
            rename_char: 6,
        }),
        "erlang" | "erl" => Some(LanguageSample {
            language_id: "erlang".to_string(),
            file_extension: ".erl".to_string(),
            content: r#"-module(main).
-export([add/2, main/0]).

add(A, B) ->
    A + B.

main() ->
    X = add(1, 2),
    X.
"#
            .to_string(),
            hover_line: 3,
            hover_char: 0,
            signature_line: 7,
            signature_char: 10,
            completion_line: 7,
            completion_char: 8,
            definition_line: 7,
            definition_char: 9,
            references_line: 3,
            references_char: 0,
            rename_line: 3,
            rename_char: 0,
        }),
        "scala" => Some(LanguageSample {
            language_id: "scala".to_string(),
            file_extension: ".scala".to_string(),
            content: r#"object Main {
  def add(a: Int, b: Int): Int = {
    a + b
  }

  def main(): Unit = {
    val x = add(1, 2)
  }
}
"#
            .to_string(),
            hover_line: 1,
            hover_char: 6,
            signature_line: 6,
            signature_char: 16,
            completion_line: 6,
            completion_char: 12,
            definition_line: 6,
            definition_char: 15,
            references_line: 1,
            references_char: 6,
            rename_line: 1,
            rename_char: 6,
        }),
        "mojo" => Some(LanguageSample {
            language_id: "mojo".to_string(),
            file_extension: ".mojo".to_string(),
            content: r#"fn add(a: Int, b: Int) -> Int:
    return a + b

fn main():
    let x = add(1, 2)
"#
            .to_string(),
            hover_line: 0,
            hover_char: 3,
            signature_line: 4,
            signature_char: 16,
            completion_line: 4,
            completion_char: 12,
            definition_line: 4,
            definition_char: 15,
            references_line: 0,
            references_char: 3,
            rename_line: 0,
            rename_char: 3,
        }),
        "pony" => Some(LanguageSample {
            language_id: "pony".to_string(),
            file_extension: ".pony".to_string(),
            content: r#"primitive Add
  fun apply(a: I32, b: I32): I32 =>
    a + b

actor Main
  new create(env: Env) =>
    let x = Add(1, 2)
"#
            .to_string(),
            hover_line: 1,
            hover_char: 6,
            signature_line: 6,
            signature_char: 16,
            completion_line: 6,
            completion_char: 12,
            definition_line: 6,
            definition_char: 15,
            references_line: 1,
            references_char: 6,
            rename_line: 1,
            rename_char: 6,
        }),
        "dart" => Some(LanguageSample {
            language_id: "dart".to_string(),
            file_extension: ".dart".to_string(),
            content: r#"int add(int a, int b) {
  return a + b;
}

void main() {
  int x = add(1, 2);
}
"#
            .to_string(),
            hover_line: 0,
            hover_char: 4,
            signature_line: 5,
            signature_char: 14,
            completion_line: 5,
            completion_char: 10,
            definition_line: 5,
            definition_char: 13,
            references_line: 0,
            references_char: 4,
            rename_line: 0,
            rename_char: 4,
        }),
        "julia" | "jl" => Some(LanguageSample {
            language_id: "julia".to_string(),
            file_extension: ".jl".to_string(),
            content: r#"function add(a::Int, b::Int)::Int
    return a + b
end

function main()
    x = add(1, 2)
    return x
end
"#
            .to_string(),
            hover_line: 0,
            hover_char: 9,
            signature_line: 5,
            signature_char: 10,
            completion_line: 5,
            completion_char: 8,
            definition_line: 5,
            definition_char: 9,
            references_line: 0,
            references_char: 9,
            rename_line: 0,
            rename_char: 9,
        }),
        "lisp" | "cl" | "commonlisp" => Some(LanguageSample {
            language_id: "lisp".to_string(),
            file_extension: ".lisp".to_string(),
            content: r#"(defun add (a b)
  (+ a b))

(defun main ()
  (let ((x (add 1 2)))
    x))
"#
            .to_string(),
            hover_line: 0,
            hover_char: 7,
            signature_line: 4,
            signature_char: 15,
            completion_line: 4,
            completion_char: 12,
            definition_line: 4,
            definition_char: 14,
            references_line: 0,
            references_char: 7,
            rename_line: 0,
            rename_char: 7,
        }),
        "fortran" | "f90" | "f95" => Some(LanguageSample {
            language_id: "fortran".to_string(),
            file_extension: ".f90".to_string(),
            content: r#"program main
    implicit none
    integer :: x
    x = add(1, 2)
contains
    function add(a, b) result(res)
        integer, intent(in) :: a, b
        integer :: res
        res = a + b
    end function add
end program main
"#
            .to_string(),
            hover_line: 5,
            hover_char: 13,
            signature_line: 3,
            signature_char: 12,
            completion_line: 3,
            completion_char: 8,
            definition_line: 3,
            definition_char: 11,
            references_line: 5,
            references_char: 13,
            rename_line: 5,
            rename_char: 13,
        }),
        "coffeescript" | "coffee" => Some(LanguageSample {
            language_id: "coffeescript".to_string(),
            file_extension: ".coffee".to_string(),
            content: r#"add = (a, b) ->
  a + b

main = ->
  x = add 1, 2
  x
"#
            .to_string(),
            hover_line: 0,
            hover_char: 0,
            signature_line: 4,
            signature_char: 8,
            completion_line: 4,
            completion_char: 6,
            definition_line: 4,
            definition_char: 7,
            references_line: 0,
            references_char: 0,
            rename_line: 0,
            rename_char: 0,
        }),
        "cython" | "pyx" => Some(LanguageSample {
            language_id: "cython".to_string(),
            file_extension: ".pyx".to_string(),
            content: r#"def add(int a, int b) -> int:
    return a + b

def main():
    cdef int x = add(1, 2)
    return x
"#
            .to_string(),
            hover_line: 0,
            hover_char: 4,
            signature_line: 4,
            signature_char: 21,
            completion_line: 4,
            completion_char: 17,
            definition_line: 4,
            definition_char: 20,
            references_line: 0,
            references_char: 4,
            rename_line: 0,
            rename_char: 4,
        }),
        "fish" => Some(LanguageSample {
            language_id: "fish".to_string(),
            file_extension: ".fish".to_string(),
            content: r#"function add
    set -l a $argv[1]
    set -l b $argv[2]
    echo (math "$a + $b")
end

function main
    set x (add 1 2)
end
"#
            .to_string(),
            hover_line: 0,
            hover_char: 9,
            signature_line: 7,
            signature_char: 13,
            completion_line: 7,
            completion_char: 11,
            definition_line: 7,
            definition_char: 12,
            references_line: 0,
            references_char: 9,
            rename_line: 0,
            rename_char: 9,
        }),
        "haxe" | "hx" => Some(LanguageSample {
            language_id: "haxe".to_string(),
            file_extension: ".hx".to_string(),
            content: r#"class Main {
    static function add(a:Int, b:Int):Int {
        return a + b;
    }

    static function main() {
        var x = add(1, 2);
    }
}
"#
            .to_string(),
            hover_line: 1,
            hover_char: 20,
            signature_line: 6,
            signature_char: 20,
            completion_line: 6,
            completion_char: 16,
            definition_line: 6,
            definition_char: 19,
            references_line: 1,
            references_char: 20,
            rename_line: 1,
            rename_char: 20,
        }),
        "holyc" => Some(LanguageSample {
            language_id: "holyc".to_string(),
            file_extension: ".hc".to_string(),
            content: r#"I64 Add(I64 a, I64 b) {
    return a + b;
}

U0 Main() {
    I64 x = Add(1, 2);
}
"#
            .to_string(),
            hover_line: 0,
            hover_char: 4,
            signature_line: 5,
            signature_char: 16,
            completion_line: 5,
            completion_char: 12,
            definition_line: 5,
            definition_char: 15,
            references_line: 0,
            references_char: 4,
            rename_line: 0,
            rename_char: 4,
        }),
        "powershell" | "ps1" => Some(LanguageSample {
            language_id: "powershell".to_string(),
            file_extension: ".ps1".to_string(),
            content: r#"function Add {
    param($a, $b)
    return $a + $b
}

function Main {
    $x = Add 1 2
    return $x
}
"#
            .to_string(),
            hover_line: 0,
            hover_char: 9,
            signature_line: 6,
            signature_char: 11,
            completion_line: 6,
            completion_char: 9,
            definition_line: 6,
            definition_char: 10,
            references_line: 0,
            references_char: 9,
            rename_line: 0,
            rename_char: 9,
        }),
        "bash" | "sh" | "shell" => Some(LanguageSample {
            language_id: "bash".to_string(),
            file_extension: ".sh".to_string(),
            content: r#"#!/bin/bash

add() {
    echo $(($1 + $2))
}

main() {
    x=$(add 1 2)
    echo $x
}
"#
            .to_string(),
            hover_line: 2,
            hover_char: 0,
            signature_line: 7,
            signature_char: 9,
            completion_line: 7,
            completion_char: 7,
            definition_line: 7,
            definition_char: 8,
            references_line: 2,
            references_char: 0,
            rename_line: 2,
            rename_char: 0,
        }),
        "raku" | "pl6" | "p6" => Some(LanguageSample {
            language_id: "raku".to_string(),
            file_extension: ".raku".to_string(),
            content: r#"sub add(Int $a, Int $b --> Int) {
    $a + $b
}

sub main() {
    my $x = add(1, 2);
    $x
}
"#
            .to_string(),
            hover_line: 0,
            hover_char: 4,
            signature_line: 5,
            signature_char: 16,
            completion_line: 5,
            completion_char: 12,
            definition_line: 5,
            definition_char: 15,
            references_line: 0,
            references_char: 4,
            rename_line: 0,
            rename_char: 4,
        }),
        "oberon" => Some(LanguageSample {
            language_id: "oberon".to_string(),
            file_extension: ".mod".to_string(),
            content: r#"MODULE Main;

PROCEDURE Add(a, b: INTEGER): INTEGER;
BEGIN
    RETURN a + b
END Add;

PROCEDURE Main;
VAR x: INTEGER;
BEGIN
    x := Add(1, 2)
END Main;

END Main.
"#
            .to_string(),
            hover_line: 2,
            hover_char: 10,
            signature_line: 10,
            signature_char: 11,
            completion_line: 10,
            completion_char: 9,
            definition_line: 10,
            definition_char: 10,
            references_line: 2,
            references_char: 10,
            rename_line: 2,
            rename_char: 10,
        }),
        "vala" => Some(LanguageSample {
            language_id: "vala".to_string(),
            file_extension: ".vala".to_string(),
            content: r#"int add(int a, int b) {
    return a + b;
}

int main() {
    int x = add(1, 2);
    return 0;
}
"#
            .to_string(),
            hover_line: 0,
            hover_char: 4,
            signature_line: 5,
            signature_char: 16,
            completion_line: 5,
            completion_char: 12,
            definition_line: 5,
            definition_char: 15,
            references_line: 0,
            references_char: 4,
            rename_line: 0,
            rename_char: 4,
        }),
        "ocaml" | "ml" => Some(LanguageSample {
            language_id: "ocaml".to_string(),
            file_extension: ".ml".to_string(),
            content: r#"let add a b =
  a + b

let main () =
  let x = add 1 2 in
  x
"#
            .to_string(),
            hover_line: 0,
            hover_char: 4,
            signature_line: 4,
            signature_char: 12,
            completion_line: 4,
            completion_char: 10,
            definition_line: 4,
            definition_char: 11,
            references_line: 0,
            references_char: 4,
            rename_line: 0,
            rename_char: 4,
        }),
        "fsharp" | "fs" => Some(LanguageSample {
            language_id: "fsharp".to_string(),
            file_extension: ".fs".to_string(),
            content: r#"let add (a: int) (b: int) : int =
    a + b

let main () =
    let x = add 1 2
    x
"#
            .to_string(),
            hover_line: 0,
            hover_char: 4,
            signature_line: 4,
            signature_char: 14,
            completion_line: 4,
            completion_char: 12,
            definition_line: 4,
            definition_char: 13,
            references_line: 0,
            references_char: 4,
            rename_line: 0,
            rename_char: 4,
        }),
        "axe" => Some(LanguageSample {
            language_id: "axe".to_string(),
            file_extension: ".axe".to_string(),
            content: r#"def add(a: i32, b: i32): i32 {
    return a + b;
}

def main() {
    val x: i32 = add(1, 2);
}
"#
            .to_string(),
            hover_line: 0,
            hover_char: 4,
            signature_line: 5,
            signature_char: 21,
            completion_line: 5,
            completion_char: 17,
            definition_line: 5,
            definition_char: 20,
            references_line: 0,
            references_char: 4,
            rename_line: 0,
            rename_char: 4,
        }),
        "ada" | "adb" | "ads" => Some(LanguageSample {
            language_id: "ada".to_string(),
            file_extension: ".adb".to_string(),
            content: r#"procedure Main is
   function Add(A, B : Integer) return Integer is
   begin
      return A + B;
   end Add;

   X : Integer;
begin
   X := Add(1, 2);
end Main;
"#
            .to_string(),
            hover_line: 1,
            hover_char: 13,
            signature_line: 8,
            signature_char: 11,
            completion_line: 8,
            completion_char: 8,
            definition_line: 8,
            definition_char: 10,
            references_line: 1,
            references_char: 13,
            rename_line: 1,
            rename_char: 13,
        }),
        "rebol" | "reb" => Some(LanguageSample {
            language_id: "rebol".to_string(),
            file_extension: ".reb".to_string(),
            content: r#"REBOL []

add: func [a [integer!] b [integer!]] [
    a + b
]

main: func [] [
    x: add 1 2
    x
]
"#
            .to_string(),
            hover_line: 2,
            hover_char: 0,
            signature_line: 7,
            signature_char: 9,
            completion_line: 7,
            completion_char: 7,
            definition_line: 7,
            definition_char: 8,
            references_line: 2,
            references_char: 0,
            rename_line: 2,
            rename_char: 0,
        }),
        "red" => Some(LanguageSample {
            language_id: "red".to_string(),
            file_extension: ".red".to_string(),
            content: r#"Red []

add: func [a [integer!] b [integer!]] [
    a + b
]

main: func [] [
    x: add 1 2
    x
]
"#
            .to_string(),
            hover_line: 2,
            hover_char: 0,
            signature_line: 7,
            signature_char: 9,
            completion_line: 7,
            completion_char: 7,
            definition_line: 7,
            definition_char: 8,
            references_line: 2,
            references_char: 0,
            rename_line: 2,
            rename_char: 0,
        }),
        "gdscript" | "gd" => Some(LanguageSample {
            language_id: "gdscript".to_string(),
            file_extension: ".gd".to_string(),
            content: r#"extends Node

func add(a: int, b: int) -> int:
    return a + b

func _ready():
    var x = add(1, 2)
"#
            .to_string(),
            hover_line: 2,
            hover_char: 5,
            signature_line: 6,
            signature_char: 16,
            completion_line: 6,
            completion_char: 12,
            definition_line: 6,
            definition_char: 15,
            references_line: 2,
            references_char: 5,
            rename_line: 2,
            rename_char: 5,
        }),
        "clojure" | "clj" | "cljs" => Some(LanguageSample {
            language_id: "clojure".to_string(),
            file_extension: ".clj".to_string(),
            content: r#"(defn add [a b]
  (+ a b))

(defn main []
  (let [x (add 1 2)]
    x))
"#
            .to_string(),
            hover_line: 0,
            hover_char: 6,
            signature_line: 4,
            signature_char: 13,
            completion_line: 4,
            completion_char: 11,
            definition_line: 4,
            definition_char: 12,
            references_line: 0,
            references_char: 6,
            rename_line: 0,
            rename_char: 6,
        }),
        "prolog" | "pro" => Some(LanguageSample {
            language_id: "prolog".to_string(),
            file_extension: ".pl".to_string(),
            content: r#"add(A, B, Result) :-
    Result is A + B.

main :-
    add(1, 2, X),
    write(X).
"#
            .to_string(),
            hover_line: 0,
            hover_char: 0,
            signature_line: 4,
            signature_char: 6,
            completion_line: 4,
            completion_char: 4,
            definition_line: 4,
            definition_char: 5,
            references_line: 0,
            references_char: 0,
            rename_line: 0,
            rename_char: 0,
        }),
        "groovy" | "gvy" => Some(LanguageSample {
            language_id: "groovy".to_string(),
            file_extension: ".groovy".to_string(),
            content: r#"def add(int a, int b) {
    return a + b
}

def main() {
    def x = add(1, 2)
    return x
}
"#
            .to_string(),
            hover_line: 0,
            hover_char: 4,
            signature_line: 5,
            signature_char: 16,
            completion_line: 5,
            completion_char: 12,
            definition_line: 5,
            definition_char: 15,
            references_line: 0,
            references_char: 4,
            rename_line: 0,
            rename_char: 4,
        }),
        "terraform" | "tf" | "hcl" => Some(LanguageSample {
            language_id: "terraform".to_string(),
            file_extension: ".tf".to_string(),
            content: r#"locals {
  sum = 1 + 2
}

variable "a" {
  type    = number
  default = 1
}

variable "b" {
  type    = number
  default = 2
}

output "result" {
  value = var.a + var.b
}
"#
            .to_string(),
            hover_line: 5,
            hover_char: 2,
            signature_line: 15,
            signature_char: 12,
            completion_line: 15,
            completion_char: 8,
            definition_line: 15,
            definition_char: 11,
            references_line: 5,
            references_char: 2,
            rename_line: 5,
            rename_char: 2,
        }),
        "factor" => Some(LanguageSample {
            language_id: "factor".to_string(),
            file_extension: ".factor".to_string(),
            content: r#"USING: math ;
IN: main

: add ( a b -- result )
    + ;

: main ( -- )
    1 2 add drop ;
"#
            .to_string(),
            hover_line: 3,
            hover_char: 2,
            signature_line: 7,
            signature_char: 10,
            completion_line: 7,
            completion_char: 8,
            definition_line: 7,
            definition_char: 9,
            references_line: 3,
            references_char: 2,
            rename_line: 3,
            rename_char: 2,
        }),
        _ => None,
    }
}

pub fn list_supported_languages() -> Vec<&'static str> {
    vec![
        "ada",
        "axe",
        "bash",
        "c",
        "clojure",
        "coffeescript",
        "cpp",
        "crystal",
        "csharp",
        "cython",
        "d",
        "dart",
        "elixir",
        "erlang",
        "factor",
        "fish",
        "fortran",
        "fsharp",
        "gdscript",
        "go",
        "groovy",
        "hare",
        "haskell",
        "haxe",
        "holyc",
        "java",
        "javascript",
        "julia",
        "kotlin",
        "lisp",
        "lua",
        "mojo",
        "nim",
        "oberon",
        "ocaml",
        "perl",
        "php",
        "pony",
        "powershell",
        "prolog",
        "python",
        "r",
        "raku",
        "rebol",
        "red",
        "ruby",
        "rust",
        "scala",
        "scheme",
        "shell",
        "swift",
        "terraform",
        "typescript",
        "vala",
        "zig",
    ]
}
