## 实现的功能

1. 兼容了ch3和ch4实现的系统调用。
2. 实现了新的系统调用spawn syscall，设置了新的 Process 调度算法—stride算法。
   - spawn 支持从当前进程创建一个新的子进程并执行输入给定的程序。
   - stride算法支持为每个task保存一个优先级(prioritiy)和当前步长(stride_pass)，进程每次被调度后步长根据优先级进行更新，并且每次 task 调度的时候选择步长最短的task进行调度。

## 问答作业

### stride 算法深入

 stride 算法原理非常简单，但是有一个比较大的问题。例如两个 pass = 10 的进程，使用 8bit 无符号整形储存 stride， p1.stride = 255, p2.stride = 250，在 p2 执行一个时间片后，理论上下一次应该 p1 执行。

 +   实际情况是轮到 p1 执行吗？为什么？
     +   不是轮到P1执行，p2执行完一个时间片之后，p2.stride再次加上stride后步幅会出现溢出的情况，8bit无符号整型上溢后会对256取余。


 我们之前要求进程优先级 >= 2 其实就是为了解决这个问题。可以证明， **在不考虑溢出的情况下** , 在进程优先级全部 >= 2 的情况下，如果严格按照算法执行，那么 STRIDE\_MAX – STRIDE\_MIN <= BigStride / 2。

 +   为什么？尝试简单说明（不要求严格证明）。
     +   初始化每个任务的 pass 为0，每次进行任务调度的时候，总是选择当前 pass 最小的任务的对pass增加。
         +   第一次调度完成后，被调度的任务的 pass 值为 BigStride / prio，其余任务的 pass 值均为0，在 prio > 2 情况下必然有 BigStride / prio - 0 <= BigStride / 2。
         +   再次进行调度，选择pass最小的任务进行调度，被调度后pass 变为一个小于等于BigStride / 2的值，此刻最大pass值减去最小pass值显然也小于BigStride / 2。
         +   以此类推，在进程优先级全部 >= 2 的情况下有STRIDE\_MAX – STRIDE\_MIN <= BigStride / 2。

 +   已知以上结论，**考虑溢出的情况下**，可以为 Stride 设计特别的比较器，让 BinaryHeap<Stride> 的 pop 方法能返回真正最小的 Stride。补全下列代码中的 `partial_cmp` 函数，假设两个 Stride 永远不会相等。

```rust
 use core::cmp::Ordering;

 struct Stride(u64);

 impl PartialOrd for Stride {
     fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
         // ...
         if (self.0 - other.0).abs > BigStride / 2 {
             if self.0 < other.0 {
                 Some(Ordering::Less)
			 } else {
             	Some(Ordering::Greater)
             }
         } else {
             if self.0 > other.0 {
                 Some(Ordering::Greater)
			} else {
             	Some(Ordering::Less)
             }
         }
     }
 }

 impl PartialEq for Stride {
     fn eq(&self, other: &Self) -> bool {
         false
     }
 }
```
> TIPS: 使用 8 bits 存储 stride, BigStride = 255, 则: `(125 < 255) == false`, `(129 < 255) == true`.


## 荣誉守则

1.  在完成本次实验的过程（含此前学习的过程）中，我曾分别与 **以下各位** 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：
    
    > 邹逸凡：小组同学，一起交流了兼容之前lab时出现的问题。
    
2.  此外，我也参考了 **以下资料** ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：
    
    > [rCore-Tutorial-Book-v3 3.6.0-alpha.1 文档](https://rcore-os.cn/rCore-Tutorial-Book-v3/index.html)
    >
    > [🐳uCore OS(on RISC-V64)实验指导书 Stride算法部分](https://nankai.gitbook.io/ucore-os-on-risc-v64/lab6/tiao-du-suan-fa-kuang-jia#stride-suan-fa)  
    
3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。
