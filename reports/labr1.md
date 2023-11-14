## 编程作业

在 `usershell` 里运行了 `42` 和 `hello` 两个用户程序。`42` 的运行结果是符合预期的，但 `hello` 的结果看起来不太对，你的任务是**修改内核，使得 `hello` 测例给出正常输出**（即给出一行以 `my name is` 开头的输出，且 `exit_code`为0）。

我把我写这道题的心路历程梳理了一下：

1. 先跑一下看看错误运行的结果是什么，跑出来的结果是`Incorrect argc`，结合`Hello.c`中的代码可以知道，是`argc`参数错误。

2. 很自然想要去打印一下 `argc` 看看它到底是啥，我们可以使用write系统调用将`&argc`打印到标准输出也就是打印到终端显示，会发现是几个'\0'加上程序中定义的常量字符，不是我们想要的结果。

3. 那么我们去分析argc的值来自哪里，看看出了什么问题。

   ```c
   int __start_main(long *p)
   {
       int argc = p[0];
       char **argv = (void *)(p+1);
   
       exit(main(argc, argv));
       return 0;
   }
   ```

   根据编写的C语言函数库可以知道，argc的值是当前用户栈顶的值，argv参数引用数组是栈中排在argc后面的一系列值。但是，在rCore的设计中，用户栈栈顶是用于对齐的空白位，接下来是参数实际的存储单元。下图反应了两种用户栈对比，左边的用户栈引用自[rCore-Tutorial-Book-v3 sys_exec 将命令行参数压入用户栈](https://rcore-os.cn/rCore-Tutorial-Book-v3/chapter7/3cmdargs-and-redirection.html#sys-exec) 。

   ![image-20231114143656919](https://jgox-image-1316409677.cos.ap-guangzhou.myqcloud.com/image-20231114143656919.png)

4. 找到问题和解决方法：在我们的c库函数设计中，是从当前用户栈栈顶获取参数个数argc，之后在argc后面的位置获取参数引用数组，和我们在rCore的设计刚好相反。那么我们想要hello.c成功运行的话，只需要修改rCore在`exec`将命令行参数压入用户栈的时候调整压入的顺序即可。我们要先压入参数存储单元，再压入参数引用数组，最后压入参数个数。具体代码实现可以从rCore的代码中找到思路。

5. 当然不用担心原本 Rust 编写的程序会出现问题，Rust 编译器约定a0 a1寄存器用来传递参数个数和参数引用数组基址，我们去维护这两个寄存器存放正确的值就可以。

6. 这里解释一下代码，写得挺有意思的。

   ```rust
   // impl TaskControlBlock {
   // pub fn exec(&self, elf_data: &[u8], args: Vec<String>) {
   // ...
   		// 预留参数存储需要的空间
           for i in 0..args.len() {
               user_sp -= args[i].len() + 1; // 加1是为了存储字符串末尾的null终止符
           }
           // 保存参数实际存储的基址
           let argv_st = user_sp;
   
           // 按照8字节对齐
           user_sp -= user_sp % core::mem::size_of::<usize>(); 
   
   		// 预留参数引用的空间
           user_sp -= (args.len() + 1) * core::mem::size_of::<usize>();
           // 保存参数引用的基址
           let argv_base = user_sp;
   
           // 获取存放参数引用的存储单元的可变引用，比如第一项是对argv[0]所在存储单元的可变引用，后续我们需要修改其中的值指向argv[0]参数真实存储的地址。
           let mut argv: Vec<_> = (0..=args.len())
               .map(|arg| {
                   translated_refmut(
                       memory_set.token(),
                       (argv_base + arg * core::mem::size_of::<usize>()) as *mut usize,
                   )
               })
               .collect();
   
           // 将user_sp指向预留的参数实际存储的存储单元
           user_sp = argv_st;
           for i in 0..args.len() {
               // 这里实现了引用指向实际存储位置。
               *argv[i] = user_sp;
               let mut p = user_sp;
               // 按字节存储参数到存储单元
               for c in args[i].as_bytes() {
                   *translated_refmut(memory_set.token(), p as *mut u8) = *c;
                   p += 1;
               }
               *translated_refmut(memory_set.token(), p as *mut u8) = 0;
               user_sp += args[i].len() + 1;
           }
   
           *argv[args.len()] = 0;
   
           // 重新将sp设置为栈顶
           user_sp = argv_base;
   
           // 在用户栈栈顶 push 参数个数
           *translated_refmut(
               memory_set.token(),
               (user_sp - core::mem::size_of::<usize>()) as *mut usize,
           ) = args.len().into();
           user_sp -= core::mem::size_of::<usize>();
   ```

7. 最后再次 `make run` 运行程序，程序成功运行。

![image-20231114144419238](https://jgox-image-1316409677.cos.ap-guangzhou.myqcloud.com/blog/image-20231114144419238.png)

## 问答作业

1. `elf` 文件和 `bin` 文件有什么区别？

- `elf`指的是**可执行和链接格式** (Executable and Linkable Format, ELF)，在 ELF 文件中， 除了程序必要的代码、数据段（它们本身都只是一些二进制的数据）之外，还有一些 **元数据** (Metadata) 描述这些段在地址空间中的位置和在 文件中的位置以及一些权限控制信息，这些元数据放在代码、数据段的外面，可以供操作系统使用。
- `bin`指的是`binary`，文件是纯粹的由二进制0和1组成的代码和数据段，是可以一行一行执行的指令，符合“存储程序按址执行”的思想。
- 分别使用`file`命令查看一个 `elf` 文件和 `bin` 文件，结果如下：

> file命令用来识别文件类型，辨别一些文件的编码格式。

![image-20231114002011535](https://jgox-image-1316409677.cos.ap-guangzhou.myqcloud.com/blog/image-20231114002011535.png)

- 对于`elf`文件，`file`命令可以根据elf文件头部的信息指出文件的类型，硬件平台等信息。
- 而对于`bin`文件，由于里面不含关于文件类型的信息，`file`命令解析不了，显示`data`。



