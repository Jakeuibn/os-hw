# lab3
## 简单总结你实现的功能（200字以内，不要贴代码）。


## 完成问答题。
stride 算法深入

stride 算法原理非常简单，但是有一个比较大的问题。例如两个 stride = 10 的进程，使用 8bit 无符号整形储存 pass， p1.pass = 255, p2.pass = 250，在 p2 执行一个时间片后，理论上下一次应该 p1 执行。

### 实际情况是轮到 p1 执行吗？为什么？
>不是，p1.pass 仍然是 255，p2.pass 是 250 + 10 = 260 % 256 = 4，p2 的 pass 更小，所以 p2 会继续执行。


我们之前要求进程优先级 >= 2 其实就是为了解决这个问题。可以证明， 在不考虑溢出的情况下 , 在进程优先级全部 >= 2 的情况下，如果严格按照算法执行，那么 PASS_MAX – PASS_MIN <= BigStride / 2。

### 为什么？尝试简单说明（不要求严格证明）。
>初始时 PASS_MAX – PASS_MIN <= BigStride / 2 满足。每次执行一个时间片后，pass == PASS_MIN 的那个 pass 会增加 BigStride / priority，而priority >= 2，所以每次增加的值至多是 BigStride / 2，也就是新的 PASS_MIN 可能的范围为旧的 PASS_MIN 至 PASS_MIN + BigStride / 2，新的 PASS_MAX 可能的范围为旧的 PASS_MAX 至 PASS_MIN + BigStride / 2，新的 PASS_MAX – PASS_MIN <= BigStride / 2仍然满足。

已知以上结论，考虑溢出的情况下，可以为 Pass 设计特别的比较器，让 BinaryHeap<Pass> 的 pop 方法能返回真正最小的 Pass。补全下列代码中的 partial_cmp 函数，假设两个 Pass 永远不会相等。
```rust
use core::cmp::Ordering;

struct Pass(u64);

impl PartialOrd for Pass {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.0 < other.0 {
            if other.0 - self.0 <= BigStride / 2 {
                Some(Ordering::Less)
            } else {
                Some(Ordering::Greater)
            }
        } else {
            if self.0 - other.0 <= BigStride / 2 {
                Some(Ordering::Greater)
            } else {
                Some(Ordering::Less)
            }
        }
    }
}

impl PartialEq for Pass {
    fn eq(&self, other: &Self) -> bool {
        false
    }
}
```
TIPS: 使用 8 bits 存储 pass, BigStride = 255, 则: (125 < 255) == false, (129 < 255) == true.


## HONOR CODE
无

## (optional) 你对本次实验设计及难度/工作量的看法，以及有哪些需要改进的地方，欢迎畅所欲言。