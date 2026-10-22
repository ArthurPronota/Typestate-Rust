# Typestate в Rust: разбор твоего примера

Твой `o_095` — учебный пример typestate, но в нём есть **одна важная деталь**, которая ломает главное обещание паттерна. Разберём.

## Что демонстрирует код

```rust
struct MyOpen;
struct MyClose;

struct MySocket<S> {
    _s: PhantomData<S>,
}

impl MySocket<MyOpen> {
    fn close(&self) -> MySocket<MyClose> { ... }
}

impl MySocket<MyClose> {
    fn open(&self) -> MySocket<MyOpen> { ... }
}
```

Идея верная:

- Состояние закодировано в **generic-параметре `S`**.
- `MySocket<MyOpen>` имеет только `close`, `MySocket<MyClose>` — только `open`.
- `PhantomData<S>` «использует» `S`, хотя поля с этим типом нет — нужно, чтобы компилятор не ругался на неиспользуемый параметр.

## ⚠️ Проблема: `main` создаёт `MyClose` напрямую

```rust
let my_cl = MySocket::<MyClose> {
    _s: std::marker::PhantomData
};
```

Это **обходит** typestate: любой код может вручную сконструировать `MySocket<MyClose>` или `MySocket<MyOpen>` в любом состоянии, потому что поля структуры **публичны** (в пределах крейта) и конструктор не ограничен. Typestate работает, только если **создать сокет в неверном состоянии невозможно**.

В твоём случае можно было бы написать:

```rust
let fake_open = MySocket::<MyOpen> { _s: PhantomData };
// и вызвать close() — хотя "открытие" никогда не происходило
```

Это не ломает компиляцию, но ломает **инвариант**: тип говорит «открыт», а реального открытия не было.

## Как сделать typestate честным

Нужно **скрыть конструктор** и сделать так, чтобы единственный способ получить `MySocket<MyOpen>` — реально открыть сокет.

### Вариант 1: приватное поле + конструктор

```rust
pub struct MySocket<S> {
    _s: PhantomData<S>,   // приватное — снаружи не создать
}

impl MySocket<MyClose> {
    pub fn new() -> Self {
        MySocket { _s: PhantomData }
    }

    pub fn open(&self) -> MySocket<MyOpen> {
        // ... реальное открытие ...
        MySocket { _s: PhantomData }
    }
}

impl MySocket<MyOpen> {
    pub fn close(&self) -> MySocket<MyClose> {
        // ... реальное закрытие ...
        MySocket { _s: PhantomData }
    }
}
```

Теперь снаружи нельзя сделать `MySocket::<MyOpen> { ... }` — поле `_s` приватное. Единственный путь к `MyOpen` — через `MySocket::new().open()`.

### Вариант 2: `Drop` для автоматического закрытия

```rust
impl Drop for MySocket<MyOpen> {
    fn drop(&mut self) {
        // закрыть сокет при выходе из области видимости
    }
}
```

Это гарантирует, что открытый сокет не «утечёт» — даже если `close()` не вызван явно.

### Вариант 3: `sealed`-паттерн для состояний

Чтобы пользователь не мог сам определить `struct MyWeird;` и подставить его в `MySocket<MyWeird>`, состояния делают «запечатанными»:

```rust
mod state {
    pub trait State {}
    pub struct Open;
    pub struct Close;
    impl State for Open {}
    impl State for Close {}
}

pub struct MySocket<S: state::State> { ... }
```

Теперь `MySocket<MyWeird>` не скомпилируется — `MyWeird` не реализует `State`, а `State` — приватный трейт.

## Почему `PhantomData` обязателен

Без него:

```rust
struct MySocket<S> {
    // нет полей с S
}
```

Компилятор выдаст:

```
error[E0392]: parameter `S` is never used
```

`PhantomData<S>` — это «нулевой» тип, который сообщает компилятору: «этот параметр используется, следи за ним». При этом он **не занимает места** в памяти (размер 0).

## Уточнения к конспекту

### «Компилятор не даст вызвать `open` на уже открытом сокете»

Это верно **только если** `open` определён лишь на `MySocket<MyClose>`. В твоём коде это так:

```rust
impl MySocket<MyClose> {
    fn open(&self) -> MySocket<MyOpen> { ... }
}
```

Вызов `my_op.open()` на `MySocket<MyOpen>` не скомпилируется — метода `open` у этого типа нет. Это и есть «бесплатный рантайм-инвариант»: проверка **на этапе компиляции**, без флагов и проверок в рантайме.

### Builder как разновидность typestate

Ты прав. Классический пример:

```rust
struct Builder<HasName, HasAge> {
    name: Option<String>,
    age: Option<u32>,
    _marker: PhantomData<(HasName, HasAge)>,
}

struct Yes;
struct No;

impl Builder<No, No> {
    fn new() -> Self { ... }
}

impl<A> Builder<No, A> {
    fn name(self, name: String) -> Builder<Yes, A> { ... }
}

impl<N> Builder<N, No> {
    fn age(self, age: u32) -> Builder<N, Yes> { ... }
}

impl Builder<Yes, Yes> {
    fn build(self) -> Person { ... }   // доступен только когда оба поля заданы
}
```

`build()` не скомпилируется, пока не вызваны `name()` и `age()`. Это typestate в чистом виде.

### Где typestate встречается

- **Сетевые протоколы**: `TcpStream<Connected>` vs `TcpStream<Disconnected>`.
- **Парсеры**: `Parser<ExpectingValue>` vs `Parser<ExpectingComma>`.
- **HTTP-клиенты**: `Request<WithBody>` vs `Request<WithoutBody>`.
- **Файлы**: `File<Open>` vs `File<Closed>`.
- **Состояния конечного автомата**: `TrafficLight<Red>` → `TrafficLight<Green>` → ...

## Практические правила

1. **Скрывай конструктор** — иначе typestate обходится.
2. **`PhantomData<S>`** — обязателен, чтобы параметр считался используемым.
3. **`sealed`-трейт** — если хочешь запретить пользовательские состояния.
4. **`Drop` для открытых состояний** — чтобы ресурс не утёк.
5. **Не переусердствуй**: typestate усложняет API и сообщения об ошибках. Применяй там, где инвариант действительно важен (протоколы, ресурсы, конечные автоматы).

## Итог

- Typestate = состояние в типе; невозможные переходы не компилируются.
- Реализуется через generic + `PhantomData` + impl-блоки на конкретных состояниях.
- В твоём примере **идея верная**, но `main` создаёт `MyClose` напрямую — это обходит инвариант. Для честного typestate нужно **скрыть конструктор** и давать состояния только через методы перехода.
- Builder — частный случай typestate (обязательные поля проверяются типом).
- Применять в API протоколов, парсеров, ресурсов, конечных автоматов.
