---
title: webAssembly和rust笔记
date: 2021-11-25 11:25:03
---

# webAssembly 与 rust

## webAssembly 历史及支持程度

为了解决js在某些场景上的性能问题(比如大量运算, 图片音视频处理等), 同时也为了让现有的大量c/c++和其他语言的功能能在浏览器环境运行, 在[2015年](https://techcrunch.com/2015/06/17/google-microsoft-mozilla-and-others-team-up-to-launch-webassembly-a-new-binary-format-for-the-web/#.xresx6:b1PE)谷歌,微软,Mozilla还有Webkit项目的开发者们开始开发一种新的能在浏览器中运行的二进制格式应用, 这就是`webAssembly`, 然后在2017年发布了第一个MVP版本, 主流浏览器都开始提供支持, 到现在的[支持情况](https://caniuse.com/wasm)是比较乐观的, 桌面端主流浏览器和移动端的ios都在2017年就提供了支持, 安卓端在最近的时候也提供了支持.
![webAssembly 支持情况](https://oss.kricsleo.com/img/webassembly.png)

## 如何运行 webAssembly

webAssembly 以二进制的格式存在, 当浏览器下载下来以后目前不能像js那样直接执行(但是未来会支持), 需要先使用[`WebAssembly`](https://developer.mozilla.org/zh-CN/docs/Web/JavaScript/Reference/Global_Objects/WebAssembly)的api解析二进制数据以后才能使用.

我们可以使用传统的xhr方式或者更新的fetch方式来下载以`.wasm`结尾的webAssembly文件

- 如果是使用xhr方式需要把下载的内容转为`ArrayBuffer`, 然后交给[`WebAssembly.instantiate()`](https://developer.mozilla.org/zh-CN/docs/Web/JavaScript/Reference/Global_Objects/WebAssembly/instantiate)解析

- 如果是使用fetch, 那么除了使用`reponse.arrayBuffer()`来把下载结果转为`ArrayBuffer`交给`WebAssembly.instantiate()`以外, 还可以使用一种更优化方式, 直接把`reponse`交给[`WebAssembly.instantiateStreaming()`](https://developer.mozilla.org/zh-CN/docs/Web/JavaScript/Reference/Global_Objects/WebAssembly/instantiateStreaming)解析, 这种方式更推荐, 解析也更快, 这种优化的方式有一个要求是`reponse`的`content-type`必须是`application/wasm`, 否则会解析失败

这个解析过程是异步的, 会返回一个promise, 当promise resolved之后会得到包含两个内容的对象: `{ module, instance }`

- `module`: 这是一个编译完成的webAssembly模块, 我们之后如果想生成更多的webAssembly实例, 就可以使用这个模块直接交给`WebAssembly.instantiate()`来得到新的实例, 避免了重复解析的过程. 这是因为`WebAssembly.instantiate()`方法有两个重载, 如果传入的`ArrayBuffer`, 那么返回的就是`{ module, instance }`, 如果传入的是之前解析完成的`module`, 那么就只会返回一个新的`instance`

- `instance`: 这是解析完成后自动生成的一个webAssembly模块实例, 包含了我们在模块中导出的方法, 可以直接使用

```js
// fetch 示例(content-type: application/wasm)
fetch('example.wasm')
  .then(response => WebAssembly.instantiateStreaming(response))
  .then(result => console.log(result)) // -> { module, instance }

// fetch 示例(content-type为其他值)
fetch('example.wasm')
  .then(response => response.arrayBuffer())
  .then(buffer => WebAssembly.instantiate(buffer))
  .then(result => console.log(result)) // -> { module, instance }
```

### WebAssembly 空间下常用函数签名

```ts
// 异步从 ArrayBuffer 或者 WebAssembly.Module 编译/实例化 Module 和 Instance
interface instantiate {
  (bufferSource: ArrayBuffer, importObj?: any): Promise<{module: WebAssembly.Module, instance: WebAssembly.Instance}>;
  (module: WebAssembly.Module, importObj?: any): Promise<WebAssembly.Instance>;
}
// 异步从 Response 编译实例化 Module 和 Instance
function instantiateStreaming(response: Response | Promise<Response>): Promise<{module: WebAssembly.Module, instance: WebAssembly.Instance}>;
// 异步从 ArrayBuffer 编译 Module
function compile(bufferSource: ArrayBuffer): Promise<WebAssembly.Module>;
// 异步从 Response 编译 Module
function compileStreaming(response: Response | Promise<Response>): Promise<WebAssembly.Module>;

// 同步从 ArrayBuffer 编译 Module(编译过程性能消耗很大, 不推荐同步)
function Module(bufferSource: ArrayBuffer): WebAssembly.Module;
// 同步从 Module 实例化 Instance(实例化过程性能消耗很大, 不推荐同步)
function Instance(module:  WebAssembly.Module, importObj?: any): WebAssembly.Instance;

// 同步验证 WebAssembly ArrayBuffer 二进制源的正确性
function validate(bufferSource: ArrayBuffer): boolean;
```

## webAssembly 可以调用外部环境的api吗?

可以.

在实例化`Instance`的参数可以传入一个`importObj`对象, 我们可以把我们想要调用的api或者对象放到这个参数里, 这个`importObj`最后会被导入到生成的实例中, 然后实例里的代码在运行的时候就可以调用我们传入的api或者数据, 注意: *WebAssembly并非直接可以调用外部环境(例如浏览器)的数据, 而是要通过这种手动显示注入的方式来把希望调用的数据传递给实例*, 注入的数据必须包含 WebAssembly 模块中已经声明好的数据格式, 否则实例化会失败.

我们可以使用一个简单的`.wasm`文件为例, *注意这里的内容是为了可读性分析, 浏览器并不能直接运行这个内容, 它仍然需要经过一层编译后才能给浏览器使用*
```plaintext
;; example.wasm
(module
  (func $i (import "k" "i") (param i32))
  (func (export "log")
    i32.const 42
    call $i))
```
`.wasm`文件一般是通过其他语言的编译工具编译的产物, 这里先不引入其他语言的编写和编译过程, 先以产物来分析 WebAssembly 本身, 把以上内容拷贝到文本编辑器中, 保存为文件例如`example.wasm`, 这样就假装得到一个编译后的产物, 分析上面的内容大概是按照`importObj`->`k`->`i`的路径找到导入的方法, 导入它并且命名为`$i`, 然后`.wasm`本身也向外部导出了一个方法叫做`log`(导入与导出的概念是不是和js的`import`和`export`很像), 在`log`中可以看到是通过`call $i`调用我们导入的方法, `.wasm`所导出的内容在`instance.exports`上, 所以我们在实例化的时候需要传入的`importObj`就是如下结构:
```js
// 被注入的对象
const importObj = {
  k: {
    i: msg => console.log(msg)
  }
};

// fetch 示例(content-type: application/wasm)
fetch('example.wasm')
  .then(response => WebAssembly.instantiateStreaming(response, importObj))
  .then(result => {
    const { instance } = result;
    instance.exports.log('hello, WebAssembly!'); // -> 'hello, WebAssembly!'
  });
```

通过这个例子可以看到 WebAssembly 是可以调用浏览器环境的api的(前提是先导入), 所以进一步来说我们也可以把操作dom的api导入到 WebAssembly 中, 这样就能在 WebAssembly 中操作dom元素, 嗯哼? 是不是已经在想着下一个高性能前端框架就用 WebAssembly 来做了? 实际上社区已经有了一些先行者开始这样做了, 比如100%支持web api的[sys-web](https://rustwasm.github.io/wasm-bindgen/web-sys/index.html)库, 还有用于前端页面开发的rust框架[Yew](https://yew.rs/), 使用rust写前端页面然后编译成 WebAssembly 运行在浏览器中, 更贴近底层的语言, 再加上多后台线程(web worker)的加持, 性能飞起, 当然了目前由于技术栈和兼容性的问题, 大规模的使用还是不太可能的, 尝尝鲜不错

## 使用 rust 编写, 然后编译为 WebAssembly

很多语言现在都可以编译到 WebAssembly, 这里以目前正在风头上的 rust 为例. 这里不会详细介绍 rust 的语法, 我们的重点在于 WebAssembly.

- 目标: 使用rust开发一个markdown的解析工具, 它最后会被编译为 WebAssembly, 然后在浏览器中运行.
- 流程: 当用户输入 markdown 内容时, 我们使用js调用`.wasm`导出的编译方法`compile()`来编译内容, `.wasm`也会调用js的`console.log()`打印自己接收到的内容和输出的内容, js拿到`compile()`编译的结果以后渲染到页面上, 一个实时预览markdown渲染结果的功能就完成了.

在rust中有现成的rust工具库来帮我们做很多事情
- [wasm-bindgen](https://crates.io/crates/wasm-bindgen)可以帮我们做自动的`import`和`export`, 就像上面那个简单的示例一样, 从外部环境导入一些我们需要的方法, 然后本身导出一些方法给外部环境调用.
- [wasm-pack](https://crates.io/crates/wasm-pack)可以帮助我们编译为 WebAssembly, 它可以打包出多种格式提供给前端开发使用, 比如直接打包出npm包, 甚至连`ts`声明文件和`readme`都帮你自动生成好, 你可以直接push到npm仓库中方便千万家. 也可以打包出比较单纯的`ES6 Module`来配合`<script type="module">`直接使用.
- 在这个例子中, 我们使用这两个工具库, 同时配合[pulldown-cmark](https://crates.io/crates/pulldown-cmark)这个使用rust编写的markdown解析工具来完成开发, 最后打包出`ES6 Module`来在页面演示效果

### rust 编写及打包

```rust
#[wasm_bindgen]
extern {
    // 这里导入浏览器环境的 console.log 到 WebAssembly 中
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log(msg: &str);
}

// 这里导出 compile 方法给浏览器调用
#[wasm_bindgen]
pub fn compile(text: &str) -> String {
    // 调用从浏览器导入的log方法打印日志
    log(text);
    let compiler = Parser::new_ext(text.trim(), Options::all());
    let mut html = String::new();
    push_html(&mut html, compiler);
    log(&html);
    return html;
}
```

rust 的部分写完了之后我们开始打包
```bash
# --target web 指定打包为 ES6 Module 内容
wasm-pack build --target web

# 默认打包的结果需要经过webpack或者其它打包工具的转译之后才能在浏览器使用
wasm-pack build

# 也可以打包为最传统的模式, 可以直接使用<script>方式引入使用
wasm-pack build --target no-modules
```
打包完成后在根目录会生成`pkg`目录, 其中包含了`markdown_compiler_bg.wasm`二进制内容和我们可以直接使用的的`markdown_compiler.js`文件.
![打包结果](https://oss.kricsleo.com/img/pkg.png)

### 页面引入使用

```html
<!-- 以 wasm-pack build --target web 打包结果为例 -->
<script type="module">
  import init, { compile } from './pkg/markdown_compiler.js';
  async function run() {
    // 初始化的过程包括下载二进制 .wasm 文件和实例化 WebAssembly
    // 初始化完成之后 compile 方法就可用了
    await init();
    const [inputNode, outputNode] = ['compiler__input', 'compiler__output'].map(t => document.getElementById(t));
    inputNode.addEventListener('input', e => outputNode.innerHTML = compile(e.target.value));
  }
  run();
</script>
```

- 示例仓库: [markdown-compiler](https://github.com/kricsleo/markdown-compiler)
- 在线演示: [markdown-compiler](https://github.kricsleo.com/markdown-compiler.html)

## 浏览器不支持 WebAssembly ?

社区里同样有工具[wasm2js](https://github.com/WebAssembly/binaryen/blob/main/src/wasm2js.h)可以把编译后的`.wasm`转为js, 不过运行速度上肯定会有一定程度下降.
