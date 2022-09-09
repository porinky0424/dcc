use std::env;

// トークンの種類
#[derive(PartialEq, Debug)]
enum TokenKind {
    Reserved,
    Num,
    EOF,
}
// トークン型
#[derive(Debug)]
struct Token {
    kind: TokenKind,
    val: Option<isize>,
    input_idx: usize, // このTokenがはじまる部分のinputされた文字列のindex
}
#[derive(Debug)]
struct TokenList {
    now: usize, // 今着目しているトークンのindex
    input: Vec<char>,
    tokens: Vec<Token>,
}

// ノードの種類
#[derive(PartialEq, Debug)]
enum NodeKind {
    ADD,
    SUB,
    MUL,
    DIV,
    NUM,
}
// ノード型
#[derive(Debug)]
struct Node {
    kind: NodeKind,
    lhs: Option<usize>, // 左辺のノードのindex
    rhs: Option<usize>, // 左辺のノードのindex
    val: Option<isize>, // kindがNUMの時のみ利用
}
#[derive(Debug)]
struct NodeList {
    nodes: Vec<Node>,
}

impl TokenList {
    fn new(p: &Vec<char>) -> Self {
        TokenList {
            now: 0,
            input: p.clone(),
            tokens: vec![],
        }
    }

    fn tokenize(p: &Vec<char>) -> Self {
        let mut token_list = Self::new(p);

        let mut idx = 0;
        while idx < p.len() {
            // 空白文字はスキップ
            if p[idx] == ' ' {
                idx += 1;
                continue;
            }

            if "+-*/()".chars().any(|c| c == p[idx]) {
                token_list.append_new_token(TokenKind::Reserved, idx, None);
                idx += 1;
                continue;
            }

            if (p[idx]).is_numeric() {
                // 数字が終わるところまでループ
                let mut digit_idx = idx + 1;
                while digit_idx < p.len() && p[digit_idx].is_numeric() {
                    digit_idx += 1;
                }
                token_list.append_new_token(
                    TokenKind::Num,
                    idx,
                    Some(
                        p[idx..digit_idx]
                            .iter()
                            .collect::<String>()
                            .parse()
                            .unwrap(),
                    ),
                );
                idx = digit_idx;
                continue;
            }

            error(idx, "tokenizeできません", p);
        }

        token_list.append_new_token(TokenKind::EOF, idx, None);
        token_list
    }

    fn get_now_token(&self) -> &Token {
        &(self.tokens[self.now])
    }

    // 次のトークンが期待している記号だったときには、トークンを1つ読み進めてtrueを返す。それ以外はfalseを返す。
    fn consume(&mut self, op: char) -> bool {
        let now_token = self.get_now_token();
        if now_token.kind != TokenKind::Reserved || self.input[now_token.input_idx] != op {
            return false;
        } else {
            self.now += 1;
            return true;
        }
    }

    // 次のトークンが期待している記号だったときには、トークンを1つ読み進める。それ以外はエラーになる。
    fn expect(&mut self, op: char) {
        let now_token = self.get_now_token();
        if now_token.kind != TokenKind::Reserved || self.input[now_token.input_idx] != op {
            error(
                now_token.input_idx,
                format!("'{}'ではありません", op).as_str(),
                &self.input,
            );
        } else {
            self.now += 1;
        }
    }

    // 次のトークンが数値の場合、トークンを1つ読み進めてその数値を返す。それ以外はエラーになる。
    fn expect_number(&mut self) -> Option<isize> {
        let now_token = self.get_now_token();
        if now_token.kind != TokenKind::Num {
            error(now_token.input_idx, "数ではありません", &self.input);
        }
        let val = now_token.val;
        self.now += 1;
        val
    }

    // fn at_eof(&self) -> bool {
    //     let now_token = self.get_now_token();
    //     now_token.kind == TokenKind::EOF
    // }

    fn append_new_token(&mut self, kind: TokenKind, input_idx: usize, val: Option<isize>) {
        self.tokens.push(Token {
            kind,
            val: if let Some(_) = val { val } else { None },
            input_idx,
        })
    }
}

impl NodeList {
    fn new() -> Self {
        NodeList { nodes: vec![] }
    }

    // 新しいノードを作成し、そのindexを返す
    fn append_new_node(&mut self, kind: NodeKind, lhs: usize, rhs: usize) -> usize {
        let new_idx = self.nodes.len();
        self.nodes.push(Node {
            kind,
            lhs: Some(lhs),
            rhs: Some(rhs),
            val: None,
        });
        new_idx
    }

    // 新しい数字ノードを作成し、そのindexを返す
    fn append_new_node_num(&mut self, val: Option<isize>, token_list: &TokenList) -> usize {
        if let None = val {
            error(
                token_list.tokens[token_list.now].input_idx,
                "数ではありません",
                &token_list.input,
            );
        }
        let new_idx = self.nodes.len();
        self.nodes.push(Node {
            kind: NodeKind::NUM,
            lhs: None,
            rhs: None,
            val,
        });
        new_idx
    }

    // expr    = mul ("+" mul | "-" mul)*
    fn expr(&mut self, token_list: &mut TokenList) -> usize {
        let mut idx = self.mul(token_list);

        loop {
            if token_list.consume('+') {
                let rhs = self.mul(token_list);
                idx = self.append_new_node(NodeKind::ADD, idx, rhs);
            } else if token_list.consume('-') {
                let rhs = self.mul(token_list);
                idx = self.append_new_node(NodeKind::SUB, idx, rhs);
            } else {
                return idx;
            }
        }
    }

    // mul     = primary ("*" primary | "/" primary)*
    fn mul(&mut self, token_list: &mut TokenList) -> usize {
        let mut idx = self.primary(token_list);

        loop {
            if token_list.consume('*') {
                let rhs = self.primary(token_list);
                idx = self.append_new_node(NodeKind::MUL, idx, rhs);
            } else if token_list.consume('/') {
                let rhs = self.primary(token_list);
                idx = self.append_new_node(NodeKind::DIV, idx, rhs);
            } else {
                return idx;
            }
        }
    }

    // primary = num | "(" expr ")"
    fn primary(&mut self, token_list: &mut TokenList) -> usize {
        if token_list.consume('(') {
            // 次のトークンが'('なら'(expr)'なはず
            let idx = self.expr(token_list);
            token_list.expect(')');
            idx
        } else {
            // そうでなければ数値なはず
            self.append_new_node_num(token_list.expect_number(), token_list)
        }
    }
}

// エラー報告用の関数
fn error(loc: usize, fmt: &str, p: &Vec<char>) {
    eprintln!("{}", p.iter().collect::<String>());
    eprintln!("{}^", " ".to_string().repeat(loc));
    eprintln!("{}", fmt);

    std::process::exit(1);
}

// NodeListからスタックマシンをemulateする形でアセンブリを出力する
fn gen(now: usize, node_list: &NodeList) {
    let now_node = &node_list.nodes[now];

    if now_node.kind == NodeKind::NUM {
        println!("  push {}", now_node.val.unwrap());
        return;
    }

    gen(now_node.lhs.unwrap(), node_list);
    gen(now_node.rhs.unwrap(), node_list);

    println!("  pop rdi");
    println!("  pop rax");

    match now_node.kind {
        NodeKind::ADD => {
            println!("  add rax, rdi");
        }
        NodeKind::SUB => {
            println!("  sub rax, rdi");
        }
        NodeKind::MUL => {
            println!("  imul rax, rdi");
        }
        NodeKind::DIV => {
            println!("  cqo");
            println!("  idiv rdi");
        }
        _ => {
            panic!("unreachable");
        }
    }

    println!("  push rax");
}

fn main() {
    let args = env::args().collect::<Vec<String>>();

    if args.len() != 2 {
        eprintln!("引数の個数が正しくありません");
        std::process::exit(1);
    }

    // 字句解析
    let mut token_list = TokenList::tokenize(&args[1].chars().collect());

    // 構文解析
    let mut node_list = NodeList::new();
    let root = node_list.expr(&mut token_list);

    // アセンブリの前半部分を出力
    println!(".intel_syntax noprefix");
    println!(".global main");
    println!("main:");

    // ASTをトップダウンに降りコード出力
    gen(root, &node_list);

    // スタックトップに残っている式の最終的な値をraxにロードして終了
    println!("  pop rax");
    println!("  ret");
}
