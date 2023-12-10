#set text(
  10pt,
  font: "Noto Serif CJK SC"
)

#show heading.where(level: 1): it => align(center, it)

= 中期文档

#align(center,
  [软件03 陈启乾 2020012385]
)

== 作业要求

+ 需要实现一种基于 GPU 的快速大规模碰撞检测算法。
+ 测试分析算法的性能。
+ 将算法应用在如下的应用中：一个固定场景中有大量小球或者物体。各个小球有不同的半径、质量、初速度和弹性系数，利用所实现的最近邻查找算法对小球或物体的运动和碰撞进行仿真，制作一段动画。（动画制作可以自己完成，也可以使用现有的软件进行渲染。）

== 说明碰撞检测的加速算法设计

朴素的碰撞检测算法中，我们会对所有物体两两之间进行碰撞检测，这样的话需要进行 $O(n^2)$ 次碰撞检测。在 $n$ 很大的时候，这样的算法效率会比较低。通过一个两阶段的碰撞检测算法

+ 粗检测阶段：在这一阶段中，我们希望能够通过较为简单、易于维护的数据结构，尽可能减少需要进行检测的物体对的数量
+ 细检测阶段：在这一阶段中，我们对粗检测阶段中判断对进行精确的碰撞检测，并用 GPU 进行并行化加速

=== 粗检测阶段

=== 细检测阶段

== 说明 GPU 实现的思路设计

本项目打算采用基于 WebGPU 技术，使用 Rust 语言的 wgpu 库实现跨平台的 GPU 程序。




== 参考文献

+ R. Weller, “A Brief Overview of Collision Detection,” in New Geometric Data Structures for Collision Detection and Haptics, R. Weller, Ed., in Springer Series on Touch and Haptic Systems. , Heidelberg: Springer International Publishing, 2013, pp. 9–46. doi: 10.1007/978-3-319-01020-5_2.
+ 用 39 行 Taichi 代码加速 GPU 粒子碰撞检测: https://zhuanlan.zhihu.com/p/563182093
+ 空间划分算法优化碰撞检测研究: https://blog.csdn.net/yhn19951008/article/details/119899092
