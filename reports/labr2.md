## 编程作业
1. 跟随前面文档的指引修改内核，使得运行测例 `hellostd` 可以输出 `Unsupported syscall_id: 29`。
   不过除了文档中提到的以外，还需要去修改`task`中的`exec`进行初始化用户栈的部分，将我们在`lab1`写的代码替换成下面这个。

   ```rust
   let elf_loader = ElfLoader::new(elf_data);
   user_sp = elf_loader.unwrap().init_stack(memory_set.token(), user_sp, args);
   ```
   传递`argc`和`argv`的寄存器值也相应的修改一下，按照`loader`模块的设计栈顶参数个数之后的就是参数引用数组。
   
   ```rust
   trap_cx.x[10] = len;
   trap_cx.x[11] = user_sp + 8;
   ```
   
2. 根据 通过 `Manual Page` 添加 `Syscall` 一节中介绍的方法，添加其他 `syscall`，使得测例 `hellostd` 可以正常运行并输出 `hello std`。（提示：只有一个新的 `syscall` 需要实现，其他的都可以用内核中现有 `syscall` 代替或者什么都不做直接返回 0）

   需要补充的第一个系统调用是 29 号系统调用，我们在[RISC-V syscall table](https://jborza.com/post/2021-05-11-riscv-linux-syscalls/) 里面搜索发现是`SYSCALL_IOCTL`，介绍如下，比较棘手，涉及到对`request code`进行分发，我们之前从来没有设计过所以先跳过这个系统调用，先试着简单的返回 0 即可。

   ```
   SYNOPSIS         top
          #include <sys/ioctl.h>
   
          int ioctl(int fd, unsigned long request, ...);
   DESCRIPTION         top
          The ioctl() system call manipulates the underlying device
          parameters of special files.  In particular, many operating
          characteristics of character special files (e.g., terminals) may
          be controlled with ioctl() requests.  The argument fd must be an
          open file descriptor.
   
          The second argument is a device-dependent request code.  The
          third argument is an untyped pointer to memory.  It's
          traditionally char *argp (from the days before void * was valid
          C), and will be so named for this discussion.
   
          An ioctl() request has encoded in it whether the argument is an
          in parameter or out parameter, and the size of the argument argp
          in bytes.  Macros and defines used in specifying an ioctl()
          request are located in the file <sys/ioctl.h>.
   ```

   需要补充的第二个系统调用查表得到是 `writev` 系统调用，函数接口是`ssize_t writev(int fd, const struct iovec *iov, int iovcnt);`，函数设计如下：

   功能：

   - 从多个缓冲区中写数据到文件中，每个缓冲区的信息由一个 `iovec` 结构体给出。

   参数包括：

   - `fd` 文件描述符
   - `iov`  结构体指针，指向一个 `iovec` 结构体数组，是需要写到文件中的缓冲区数组。
   - `iovcnt` 缓冲区的长度，也就是有多少个缓冲区需要写入到文件

   返回值：

   - 成功返回写入的字节数
   - 失败返回 -1

   到这里还差一步是 `iovec` 结构体的信息，其实在网站中也有，比较隐晦，放在 `example` 中给出了 `iovec` 结构体成员变量 `iov_base iov_len`。

   最后是代码实现，按照功能设计很好实现，循环多次写数据即可，需要注意：

   - 写入数据到文件的时候可以使用 `sys_write` 写，不想处理 `sys_write` 的返回值和频繁调用开销大我们直接使用 `file.write` 去写。
   - 怎么遍历缓冲区数组的问题？遍历数组很简单 `for` 循环更新访问地址即可，但问题是我们拿到的是在用户页表中建立了映射的虚拟地址，我们需要访问数组中的元素内存管理模块提供 `translated_ref` 方法，可以通过变量的虚拟地址和该变量所在用户页表的根目录地址获取到该变量的引用，那么就很简单了，代码实现如下：

   ```rust
   pub fn sys_writev(fd: usize, iov: *mut Iovec, iovcnt: usize) -> isize {
       let mut total_write_size: isize = 0;
       let token = current_user_token();
       let task = current_task().unwrap();
       let inner = task.inner_exclusive_access();
       if fd >= inner.fd_table.len() {
           return -1;
       }
       if let Some(file) = &inner.fd_table[fd] {
           let file = file.clone();
           drop(inner);
           for i in 0..iovcnt {
               let iov_ptr = translated_ref(token, iov.wrapping_add(i));
               total_write_size += file.write(UserBuffer::new(translated_byte_buffer(
                   token,
                   (*iov_ptr).iov_base,
                   (*iov_ptr).iov_len,
               ))) as isize;
           }
       } else {
           return -1;
       }
       total_write_size
   }
   ```

   最后发现还缺少一个系统调用 `SYSCALL_EXIT_GROUP` ，用来终止调用进程线程组中的所有线程，我们这里没用到线程，直接换成 `sys_exit` 试试。

   ```rust
   SYSCALL_EXIT_GROUP => sys_exit(0),
   ```

   再次运行，成功得到了正确输出。😊

   ![image-20231119212235445](https://jgox-image-1316409677.cos.ap-guangzhou.myqcloud.com/blog/image-20231119212235445.png)

## 问答作业

1. 查询标志位定义。

标准的 `waitpid` 调用的结构是 `pid_t waitpid(pid_t pid, int *_Nullable wstatus, int options);`。其中的 `options` 参数分别有哪些可能（只要列出不需要解释），用 `int` 的 32 个 bit 如何表示？

在编辑器中全局搜索wait_pid不难找到对于这个函数的解释。

![image-20231119210254222](https://jgox-image-1316409677.cos.ap-guangzhou.myqcloud.com/blog/image-20231119210254222.png)

我们再全局搜索WNOHANG，可以找到下面的宏定义。

```c
// /include/stdlib.h
#define WNOHANG    1
#define WUNTRACED  2
```

所以option可能的参数如上。用 `int` 的 32 个 bit 表示即分别为`0x00000001` `0x00000002`。



