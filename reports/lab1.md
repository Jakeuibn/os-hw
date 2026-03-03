# lab1: System Call Tracing
## 简单总结你实现的功能（200字以内，不要贴代码）。
系统调用跟踪功能的实现主要包括以下几个方面：
1. 在 `TaskControlBlock` 结构体中添加一个新的字段 `syscall_count`，用于记录每个系统调用的调用次数。
2. 在系统调用处理函数 `syscall` 中调用 `trace_current_syscall_count` 函数，更新当前系统调用的计数。
3. 实现 `sys_trace` 系统调用，根据传入的 `trace_request` 参数执行不同的操作：
   - 当 `trace_request` 为 0时，读取指定地址的一个字节并返回。
   - 当 `trace_request` 为 1时，向指定地址写入一个字节。
   - 当 `trace_request` 为 2时，返回当前系统调用的调用次数。
4. 在用户程序中调用 `sys_trace` 系统调用，验证系统调用跟踪功能的正确性。

## 完成问答题。
### 1. 正确进入 U 态后，程序的特征还应有：使用 S 态特权指令，访问 S 态寄存器后会报错。 请同学们可以自行测试这些内容（运行 三个 bad 测例 (ch2b_bad_*.rs) ）， 描述程序出错行为，同时注意注明你使用的 sbi 及其版本。
行为：

PageFault in application, bad addr = 0x0, bad instruction = 0x804003a4, kernel killed it.

IllegalInstruction in application, kernel killed it.

IllegalInstruction in application, kernel killed it.

会在错误的指令运行时产生异常，内核会杀死应用程序。

版本：

RustSBI version 0.3.0-alpha.2, adapting to RISC-V SBI v1.0.0

sbi版本为0.3.0-alpha.2，适配RISC-V SBI v1.0.0。

### 2. 深入理解 trap.S 中两个函数 __alltraps 和 __restore 的作用，并回答如下问题:
1. L40：刚进入 __restore 时，sp 代表了什么值。请指出 __restore 的两种使用情景。

对应进程的kernel stack的栈顶地址。__restore的两种使用情景分别是：trap后从内核态恢复到用户态继续执行，以及第一次开始执行该app

2. L43-L48：这几行汇编代码特殊处理了哪些寄存器？这些寄存器的的值对于进入用户态有何意义？请分别解释。
```
ld t0, 32*8(sp)
ld t1, 33*8(sp)
ld t2, 2*8(sp)
csrw sstatus, t0
csrw sepc, t1
csrw sscratch, t2
```
这几行代码分别加载了保存在栈上的sstatus、sepc的值，并将它们写回对应的寄存器中，利用sscratch设置用户栈的sp。sstatus寄存器保存了当前的状态信息，包括特权级别等，告诉CPU应该回到哪个特权级别；sepc寄存器保存了异常发生时的程序计数器（PC），告诉CPU程序从哪里继续执行；sscratch寄存器用于保存user stack栈顶，方便后面设置sp。

3. L50-L56：为何跳过了 x2 和 x4？
```
ld x1, 1*8(sp)
ld x3, 3*8(sp)
.set n, 5
.rept 27
   LOAD_GP %n
   .set n, n+1
.endr
```
x2寄存器是sp，x4寄存器是tp，sp最后再通过sscratch保存的值设置，避免sp的值修改导致无法load别的值，x4寄存器用于保存线程指针，这里没有用到。

4. L60：该指令之后，sp 和 sscratch 中的值分别有什么意义？

csrrw sp, sscratch, sp

sp是user stack的栈顶地址，sscratch中保存了原来sp的值，即内核栈的栈顶地址。

5. __restore：中发生状态切换在哪一条指令？为何该指令执行之后会进入用户态？

sret指令。sret指令会根据sstatus寄存器中的特权级别信息，切换到对应的特权级别，这里是用户态（U态），这是CPU做的。

6. L13：该指令之后，sp 和 sscratch 中的值分别有什么意义？

csrrw sp, sscratch, sp

sp是内核栈的栈顶地址，sscratch中保存了原来sp的值，即user stack的栈顶地址。

7. 从 U 态进入 S 态是哪一条指令发生的？
ecall，或者产生中断异常时的指令

## HONOR CODE
无

## (optional) 你对本次实验设计及难度/工作量的看法，以及有哪些需要改进的地方，欢迎畅所欲言。