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
        format!("file:///tmp/lsp_health_check{}", self.file_extension)
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
            hover_line: 2,
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
        _ => None,
    }
}
