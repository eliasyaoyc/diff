/**
### 核心的trie树
这个算是整个系统稍微复杂一点的部分
核心就是一个Trie树
node是一个trie树
每个节点都是以.分割的字符串
foo.bar.aa
foo.cc.aa
foo.bb.dd
foo.dd
```
               foo
    /    /  |  \   \    \
    *    >  bar cc  bb  dd
    |			|   |   |
    aa	   aa	aa  aa
    ```
    当一个订阅foo.> 插入这个树上的时候, 这个订阅会放到>中去 ,称之为sub1
    当一个foo.* 插入的时候,订阅会放到* sub2
    当一个订阅foo.bar.aa 订阅来的时候会放到foo.bar.aa中去 sub3
    当有人再foo.ff 发布一个消息的时候会匹配到sub1,sub2
    当有人再foo.bar.aa发布一个消息的时候会匹配到sub2,sub3
    ### cache系统
    每次查找虽然是LogN,但是代价也挺大的,因此搞了缓存
    一个trie树遍历的缓存,当一个publisher发表一个消息的时候,很可能会针对这个主题再次发布消息,
    那么查找到的相关的所有的subscriber,可以缓存起来
    负面: 当新增或者删除subscriber的时候也要来cache里面遍历,修改.
*/
use crate::error::*;
use crate::simple_sublist::*;

const PWC: &str = "*";
const FWC: &str = ">";

pub struct Level{

}