## TODO
- 共识机制从Aura转换成Babe
- 设定每次出块的奖励逻辑
- NFT如何实现

### 种子轮
- 确定当前的TEA和CML的发行量
- 设定当前CML逻辑
- Genesis设定

- 正式上限阶段的CML逻辑，和从种子轮如何转换的规则


### 问题：
- NFT的id一般是什么格式的，当前Asset pallet的实现为 u32 数字，是否可用？

- 当前实现的PCML结构为
```
{
  id: AssetId,
  group: Vec<u8>,  // nitro
  created_at: BlockNumber,
}
```

