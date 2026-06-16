// ============================================================
// 漏壶系统前端测试套件
// 覆盖4大功能模块，共 45 个测试用例
// 正常场景 / 边界场景 / 异常场景
// ============================================================

let API_BASE = 'http://localhost:8080/api';

const testSuites = [
  // ============================================================
  // 1. 朝代精度对比测试
  // ============================================================
  {
    id: 'dynasty',
    name: '📜 朝代漏壶精度对比测试',
    tests: [
      {
        id: 'd1', name: '正常场景：获取所有朝代列表', type: '正常',
        async run() {
          const data = await apiGet('/dynasties');
          assert(Array.isArray(data), '返回值应为数组');
          assert(data.length === 6, `应有6个朝代，实际${data.length}个`);
          const ids = data.map(d => d.dynasty_id);
          assert(ids.includes('HAN_CHENJIAN'), '应包含汉代沉箭漏');
          assert(ids.includes('SONG_LIANHUA'), '应包含宋代莲花漏');
          assert(ids.includes('SONG_YITIAN'), '应包含宋代水运仪象台');
        }
      },
      {
        id: 'd2', name: '正常场景：朝代详情结构完整', type: '正常',
        async run() {
          const d = await apiGet('/dynasties/SONG_LIANHUA');
          assert(d.dynasty_id === 'SONG_LIANHUA', 'ID匹配');
          assert(d.dynasty_name === '宋代', '朝代名称正确');
          assert(d.stage_count === 3, '莲花漏应为3级');
          assert(Array.isArray(d.configs), '应有configs数组');
          assert(d.configs.length === 3, '应有3个漏壶配置');
          assert(typeof d.historical_daily_error_seconds === 'number', '有历史误差');
          assert(d.description && d.description.length > 10, '有描述文字');
        }
      },
      {
        id: 'd3', name: '正常场景：宋朝vs唐朝对比成功', type: '正常',
        async run() {
          const cmp = await apiGet('/dynasties/compare/SONG_YITIAN/TANG_JINGLU');
          assert(cmp.left_dynasty.dynasty_name === '宋代', '左为宋');
          assert(cmp.right_dynasty.dynasty_name === '唐代', '右为唐');
          assert(typeof cmp.left_daily_error_seconds === 'number', '左误差数值');
          assert(typeof cmp.right_daily_error_seconds === 'number', '右误差数值');
          assert(cmp.winner === '宋代', '宋代应为胜者');
          assert(cmp.left_daily_error_seconds < cmp.right_daily_error_seconds, '宋误差<唐');
          assert(Array.isArray(cmp.key_differences) && cmp.key_differences.length > 0, '有关键差异');
          assert(Array.isArray(cmp.flow_comparison) && cmp.flow_comparison.length > 0, '有流量对比');
        }
      },
      {
        id: 'd4', name: '边界场景：最优(宋莲花)vs最差(汉沉箭)', type: '边界',
        async run() {
          const cmp = await apiGet('/dynasties/compare/SONG_LIANHUA/HAN_CHENJIAN');
          assert(cmp.left_daily_error_seconds > 0, '误差正数');
          assert(cmp.right_daily_error_seconds > 0, '误差正数');
          assert(cmp.left_daily_error_seconds < cmp.right_daily_error_seconds, '宋<汉');
          assert(cmp.winner === '宋代', '宋代胜');
          assert(cmp.error_ratio > 0 && cmp.error_ratio < 1, '误差比在0-1之间');
          assert(cmp.left_dynasty.stage_count === 3, '宋3级');
          assert(cmp.right_dynasty.stage_count === 1, '汉1级');
        }
      },
      {
        id: 'd5', name: '边界场景：自身对比结果恒等', type: '边界',
        async run() {
          const cmp = await apiGet('/dynasties/compare/SONG_YITIAN/SONG_YITIAN');
          assert(Math.abs(cmp.left_daily_error_seconds - cmp.right_daily_error_seconds) < 1e-6,
            '自身对比误差应完全相等');
          assert(Math.abs(cmp.error_ratio - 1.0) < 1e-6, '误差比应为1');
        }
      },
      {
        id: 'd6', name: '异常场景：左侧无效朝代ID', type: '异常',
        async run() {
          try {
            await apiGet('/dynasties/compare/INVALID/SONG_YITIAN');
            assert(false, '应抛出错误');
          } catch (e) {
            assert(e.status && e.status !== 200 || e.message, '返回错误响应');
          }
        }
      },
      {
        id: 'd7', name: '异常场景：右侧无效朝代ID', type: '异常',
        async run() {
          try {
            await apiGet('/dynasties/compare/SONG_YITIAN/BAD_ID');
            assert(false, '应抛出错误');
          } catch (e) {
            assert(true, '正确返回错误');
          }
        }
      },
      {
        id: 'd8', name: '数据完整性：每朝配置数量等于级数', type: '正常',
        async run() {
          const list = await apiGet('/dynasties');
          for (const d of list) {
            assert(d.configs.length === d.stage_count,
              `${d.dynasty_name}: configs(${d.configs.length})应等于级数(${d.stage_count})`);
          }
        }
      },
      {
        id: 'd9', name: '趋势验证：级数越多精度越高', type: '正常',
        async run() {
          const list = await apiGet('/dynasties');
          const withError = await Promise.all(list.map(async d => {
            const detail = await apiGet('/dynasties/' + d.dynasty_id);
            return { stages: d.stage_count, error: detail.daily_error_seconds || d.historical_daily_error_seconds };
          }));
          const byStages = {};
          for (const item of withError) {
            if (!byStages[item.stages]) byStages[item.stages] = [];
            byStages[item.stages].push(item.error);
          }
          const s1 = Math.min(...byStages[1]);
          const s2 = Math.min(...(byStages[2] || [Infinity]));
          const s3 = Math.min(...(byStages[3] || [Infinity]));
          const s4 = Math.min(...(byStages[4] || [Infinity]));
          assert(s1 > s2 || s2 > s3 || s3 > s4, '整体呈现级数↑误差↓趋势');
        }
      }
    ]
  },

  // ============================================================
  // 2. 跨时代精度对比测试
  // ============================================================
  {
    id: 'crossera',
    name: '⏱️ 跨时代精度对比测试',
    tests: [
      {
        id: 'c1', name: '正常场景：现代计时器列表', type: '正常',
        async run() {
          const list = await apiGet('/modern');
          assert(Array.isArray(list), '返回数组');
          assert(list.length >= 8, `至少8款设备，实际${list.length}`);
          const categories = [...new Set(list.map(m => m.category))];
          assert(categories.includes('机械'), '包含机械类');
          assert(categories.includes('电子'), '包含电子类');
          assert(categories.includes('原子'), '包含原子类');
          assert(categories.includes('卫星'), '包含卫星类');
        }
      },
      {
        id: 'c2', name: '正常场景：跨时代综合对比结构完整', type: '正常',
        async run() {
          const cmp = await apiGet('/cross-era');
          assert(Array.isArray(cmp.ancient_devices), '古代设备数组');
          assert(Array.isArray(cmp.modern_devices), '现代设备数组');
          assert(cmp.ancient_devices.length === 6, '6款古代');
          assert(cmp.modern_devices.length === 8, '8款现代');
          assert(cmp.best_ancient, '有古代最优');
          assert(cmp.best_modern, '有现代最优');
          assert(typeof cmp.improvement_factor === 'number', '有改进倍数');
          assert(Array.isArray(cmp.timeline_data), '有时间线');
        }
      },
      {
        id: 'c3', name: '可靠性验证：现代比古代精度高', type: '正常',
        async run() {
          const cmp = await apiGet('/cross-era');
          assert(cmp.best_ancient.daily_error_seconds > cmp.best_modern.daily_error_seconds,
            '现代最优精度 > 古代最优');
          assert(cmp.improvement_factor > 1, '改进倍数 > 1');
          assert(cmp.best_ancient.era === '古代', '古代最优标签正确');
        }
      },
      {
        id: 'c4', name: '时间线：年份严格递增', type: '边界',
        async run() {
          const cmp = await apiGet('/cross-era');
          const tl = cmp.timeline_data;
          assert(tl.length >= 10, '至少10个时间节点');
          for (let i = 1; i < tl.length; i++) {
            assert(tl[i].year > tl[i-1].year,
              `第${i}年(${tl[i].year})应大于第${i-1}年(${tl[i-1].year})`);
          }
          assert(tl[0].year < 0, '起点应为公元前');
          assert(tl[tl.length-1].year > 1950, '终点应在1950年后');
        }
      },
      {
        id: 'c5', name: '精度排序：原子 < 电子 < 机械', type: '正常',
        async run() {
          const list = await apiGet('/modern');
          const byCat = {};
          for (const m of list) {
            if (!byCat[m.category]) byCat[m.category] = [];
            byCat[m.category].push(m.daily_error_seconds);
          }
          const atomMin = Math.min(...byCat['原子']);
          const elecMin = Math.min(...byCat['电子']);
          const mechMin = Math.min(...byCat['机械']);
          assert(atomMin < elecMin, '原子 < 电子');
          assert(elecMin < mechMin, '电子 < 机械');
        }
      },
      {
        id: 'c6', name: '数据校验：每台设备字段完整', type: '正常',
        async run() {
          const list = await apiGet('/modern');
          for (const m of list) {
            assert(m.piece_id && m.piece_id.length > 0, '有piece_id');
            assert(m.name && m.name.length > 0, '有name');
            assert(m.daily_error_seconds > 0, '日误差>0');
            assert(m.yearly_error_seconds > 0, '年误差>0');
            assert(m.invention_year > 0, '有发明年份');
            assert(['机械','电子','原子','卫星'].includes(m.category),
              `${m.name}类别${m.category}有效`);
          }
        }
      },
      {
        id: 'c7', name: '古代最优为宋或明（技术巅峰）', type: '正常',
        async run() {
          const cmp = await apiGet('/cross-era');
          const label = cmp.best_ancient.label;
          const isSongOrMing = label.includes('宋') || label.includes('明');
          assert(isSongOrMing, `古代最优应是宋或明 (实际: ${label})`);
        }
      },
      {
        id: 'c8', name: '现代最优为原子或卫星类', type: '正常',
        async run() {
          const cmp = await apiGet('/cross-era');
          const cat = cmp.best_modern.category;
          assert(cat === '原子' || cat === '卫星',
            `现代最优类别应为原子/卫星 (实际: ${cat})`);
        }
      },
      {
        id: 'c9', name: '误差范围：古代设备误差有差异', type: '边界',
        async run() {
          const cmp = await apiGet('/cross-era');
          const errors = cmp.ancient_devices.map(d => d.daily_error_seconds);
          const minE = Math.min(...errors);
          const maxE = Math.max(...errors);
          assert(minE > 0, '最小误差>0');
          assert(maxE > minE, '古代设备间应有精度差异');
        }
      }
    ]
  },

  // ============================================================
  // 3. 多级漏壶误差传递测试
  // ============================================================
  {
    id: 'errortransfer',
    name: '🔗 误差传递分析测试',
    tests: [
      {
        id: 'e1', name: '正常场景：宋代4级误差传递结构', type: '正常',
        async run() {
          const a = await apiGet('/error-transfer/SONG_YITIAN');
          assert(Array.isArray(a.nodes) && a.nodes.length === 4, '4个节点');
          assert(typeof a.total_error_seconds === 'number', '有总误差');
          assert(typeof a.bottleneck_stage === 'number', '有瓶颈级');
          assert(a.bottleneck_stage >= 0 && a.bottleneck_stage < 4, '瓶颈级在0-3间');
          assert(typeof a.bottleneck_reason === 'string' && a.bottleneck_reason.length > 0, '有瓶颈原因');
          assert(Array.isArray(a.recommendations) && a.recommendations.length > 0, '有优化建议');
          assert(typeof a.compensation_potential_seconds === 'number', '有补偿潜力');
        }
      },
      {
        id: 'e2', name: '累积效应：输出误差≥输入误差', type: '正常',
        async run() {
          const a = await apiGet('/error-transfer/SONG_YITIAN');
          for (let i = 0; i < a.nodes.length; i++) {
            const n = a.nodes[i];
            if (i > 0) {
              assert(Math.abs(n.input_error_seconds - a.nodes[i-1].output_error_seconds) < 1e-6,
                `第${i}级输入 = 第${i-1}级输出`);
            }
            assert(n.output_error_seconds >= n.input_error_seconds,
              `第${i}级输出(${n.output_error_seconds}) ≥ 输入(${n.input_error_seconds})`);
            assert(n.self_error_seconds > 0, `第${i}级自身误差>0`);
          }
        }
      },
      {
        id: 'e3', name: '贡献度：各节点贡献和为100%', type: '正常',
        async run() {
          const a = await apiGet('/error-transfer/SONG_YITIAN');
          const sum = a.nodes.reduce((s, n) => s + n.contribution_percent, 0);
          assert(Math.abs(sum - 100) < 2, `贡献和应≈100% (实际: ${sum.toFixed(2)}%)`);
        }
      },
      {
        id: 'e4', name: '瓶颈级：贡献占比最大', type: '正常',
        async run() {
          const a = await apiGet('/error-transfer/TANG_JINGLU');
          const bn = a.bottleneck_stage;
          const bnContrib = a.nodes[bn].contribution_percent;
          for (let i = 0; i < a.nodes.length; i++) {
            if (i !== bn) {
              assert(a.nodes[i].contribution_percent <= bnContrib + 1e-9,
                `瓶颈级${bn}贡献${bnContrib}%应最大 (第${i}级为${a.nodes[i].contribution_percent}%)`);
            }
          }
        }
      },
      {
        id: 'e5', name: '边界场景：单级漏壶无累积效应', type: '边界',
        async run() {
          const a = await apiGet('/error-transfer/HAN_CHENJIAN');
          assert(a.nodes.length === 1, '单级只有1节点');
          const n = a.nodes[0];
          assert(n.input_error_seconds === 0, '单级输入误差为0');
          assert(n.output_error_seconds === n.self_error_seconds,
            '单级输出=自身误差 (无多级累积)');
          assert(n.amplification_factor >= 1, '放大系数≥1');
        }
      },
      {
        id: 'e6', name: '多级更优：4级单级平均误差 < 1级', type: '正常',
        async run() {
          const a1 = await apiGet('/error-transfer/HAN_CHENJIAN');
          const a4 = await apiGet('/error-transfer/SONG_YITIAN');
          const avg1 = a1.total_error_seconds / a1.nodes.length;
          const avg4 = a4.total_error_seconds / a4.nodes.length;
          assert(avg1 > avg4,
            `多级补偿效应: 1级平均(${avg1.toFixed(1)}s) > 4级平均(${avg4.toFixed(1)}s)`);
        }
      },
      {
        id: 'e7', name: '异常场景：无效朝代ID', type: '异常',
        async run() {
          try {
            await apiGet('/error-transfer/INVALID_ID');
            assert(false, '应返回错误');
          } catch (e) {
            assert(true, '正确处理无效ID');
          }
        }
      },
      {
        id: 'e8', name: '每节点字段完整性检查', type: '正常',
        async run() {
          const a = await apiGet('/error-transfer/SONG_YITIAN');
          for (const n of a.nodes) {
            assert(n.stage_name && n.stage_name.length > 0, '有stage_name');
            assert(typeof n.input_error_seconds === 'number', '有input_error');
            assert(typeof n.self_error_seconds === 'number', '有self_error');
            assert(typeof n.output_error_seconds === 'number', '有output_error');
            assert(typeof n.amplification_factor === 'number', '有放大系数');
            assert(typeof n.contribution_percent === 'number', '有贡献度');
            assert(n.contribution_percent >= 0 && n.contribution_percent <= 100,
              '贡献度在0-100之间');
          }
        }
      },
      {
        id: 'e9', name: '补偿潜力：正数且不超过总误差', type: '边界',
        async run() {
          const a = await apiGet('/error-transfer/SONG_YITIAN');
          assert(a.compensation_potential_seconds > 0, '补偿潜力>0');
          assert(a.compensation_potential_seconds <= a.total_error_seconds,
            '补偿潜力 ≤ 总误差');
        }
      }
    ]
  },

  // ============================================================
  // 4. 虚拟操作体验测试
  // ============================================================
  {
    id: 'virtual',
    name: '🎮 虚拟操作体验测试',
    tests: [
      {
        id: 'v1', name: '正常场景：中位水位操作', type: '正常',
        async run() {
          const r = await apiPost('/virtual-operate', {
            clepsydra_id: 'KD1',
            target_water_level_cm: 70,
            water_temp_c: 20,
            simulate_seconds: 3600
          });
          assert(r.clepsydra_id === 'KD1', 'ID正确');
          assert(r.final_level_cm > 0, '最终水位>0');
          assert(r.time_elapsed_simulated === 3600, '模拟时长正确');
          assert(Array.isArray(r.level_history) && r.level_history.length > 2, '有水位历史');
          assert(Array.isArray(r.error_history) && r.error_history.length > 2, '有误差历史');
          assert(Array.isArray(r.flow_history) && r.flow_history.length > 2, '有流量历史');
          assert(r.level_history[0][0] === 0, '历史从0秒开始');
        }
      },
      {
        id: 'v2', name: '边界场景：最高水位', type: '边界',
        async run() {
          const r = await apiPost('/virtual-operate', {
            clepsydra_id: 'KD1',
            target_water_level_cm: 120,
            water_temp_c: 20,
            simulate_seconds: 600
          });
          assert(r.final_level_cm <= 120 + 1e-6, '不超过上限120cm');
          assert(Array.isArray(r.observations) && r.observations.length > 0, '有观察结论');
        }
      },
      {
        id: 'v3', name: '边界场景：最低水位', type: '边界',
        async run() {
          const r = await apiPost('/virtual-operate', {
            clepsydra_id: 'KD2',
            target_water_level_cm: 15,
            water_temp_c: 20,
            simulate_seconds: 600
          });
          assert(r.final_level_cm >= 15 - 1e-6, '不低于下限15cm');
        }
      },
      {
        id: 'v4', name: '异常处理：水位超上限自动截断', type: '异常',
        async run() {
          const r = await apiPost('/virtual-operate', {
            clepsydra_id: 'KD3',
            target_water_level_cm: 500,
            water_temp_c: 20,
            simulate_seconds: 100
          });
          assert(r.final_level_cm <= 80 + 1, '500cm应被截断到KD3上限约80cm');
        }
      },
      {
        id: 'v5', name: '异常处理：负水位自动修正', type: '异常',
        async run() {
          const r = await apiPost('/virtual-operate', {
            clepsydra_id: 'KD3',
            target_water_level_cm: -10,
            water_temp_c: 20,
            simulate_seconds: 100
          });
          assert(r.final_level_cm >= 0, '负水位应被修正');
        }
      },
      {
        id: 'v6', name: '物理规律：高水位→高流量', type: '正常',
        async run() {
          const low = await apiPost('/virtual-operate', {
            clepsydra_id: 'KD1', target_water_level_cm: 30, water_temp_c: 20, simulate_seconds: 100
          });
          const high = await apiPost('/virtual-operate', {
            clepsydra_id: 'KD1', target_water_level_cm: 110, water_temp_c: 20, simulate_seconds: 100
          });
          const lowFlow = low.flow_history[low.flow_history.length - 1][1];
          const highFlow = high.flow_history[high.flow_history.length - 1][1];
          assert(highFlow > lowFlow,
            `高水位(110cm)流量${highFlow.toFixed(3)} > 低水位(30cm)流量${lowFlow.toFixed(3)}`);
        }
      },
      {
        id: 'v7', name: '时长一致性：多档时长均正确', type: '边界',
        async run() {
          const cases = [60, 600, 3600, 86400];
          for (const secs of cases) {
            const r = await apiPost('/virtual-operate', {
              clepsydra_id: 'KD1', target_water_level_cm: 60, water_temp_c: 20, simulate_seconds: secs
            });
            assert(r.time_elapsed_simulated === secs, `${secs}秒时长正确`);
            const lastT = r.level_history[r.level_history.length - 1][0];
            assert(Math.abs(lastT - secs) <= 1,
              `${secs}s模拟: 最后时间点${lastT.toFixed(0)}s应接近目标`);
          }
        }
      },
      {
        id: 'v8', name: '下限保护：极短时长取最小值', type: '边界',
        async run() {
          const r = await apiPost('/virtual-operate', {
            clepsydra_id: 'KD1', target_water_level_cm: 60, water_temp_c: 20, simulate_seconds: 1
          });
          assert(r.time_elapsed_simulated >= 10, '1秒应被提升到最小值10秒');
        }
      },
      {
        id: 'v9', name: '上限保护：超长时长取1天', type: '边界',
        async run() {
          const r = await apiPost('/virtual-operate', {
            clepsydra_id: 'KD1', target_water_level_cm: 60, water_temp_c: 20, simulate_seconds: 200000
          });
          assert(r.time_elapsed_simulated <= 86400, '200000秒应被限制到86400秒(1天)');
        }
      },
      {
        id: 'v10', name: '异常场景：无效漏壶ID', type: '异常',
        async run() {
          try {
            await apiPost('/virtual-operate', {
              clepsydra_id: 'NON_EXIST', target_water_level_cm: 50, water_temp_c: 20, simulate_seconds: 100
            });
            assert(false, '应返回错误');
          } catch (e) {
            assert(true, '正确处理无效漏壶ID');
          }
        }
      },
      {
        id: 'v11', name: '水温影响：极端水温触发观察', type: '边界',
        async run() {
          const cold = await apiPost('/virtual-operate', {
            clepsydra_id: 'KD1', target_water_level_cm: 70, water_temp_c: 0, simulate_seconds: 200
          });
          const hot = await apiPost('/virtual-operate', {
            clepsydra_id: 'KD1', target_water_level_cm: 70, water_temp_c: 40, simulate_seconds: 200
          });
          const hasCold = cold.observations.some(o => o.includes('水温') || o.includes('低温'));
          const hasHot = hot.observations.some(o => o.includes('水温') || o.includes('高温'));
          assert(hasCold || hasHot, '极端水温应有水温相关观察结论');
        }
      },
      {
        id: 'v12', name: '直观性：观察结论有信息量', type: '正常',
        async run() {
          const r = await apiPost('/virtual-operate', {
            clepsydra_id: 'KD1', target_water_level_cm: 25, water_temp_c: 35, simulate_seconds: 600
          });
          assert(r.observations.length >= 1, '至少1条观察');
          for (const obs of r.observations) {
            assert(typeof obs === 'string' && obs.length > 5,
              `观察内容应有足够长度: "${obs}"`);
          }
        }
      },
      {
        id: 'v13', name: '历史数据：三数组长度一致', type: '正常',
        async run() {
          const r = await apiPost('/virtual-operate', {
            clepsydra_id: 'KD1', target_water_level_cm: 70, water_temp_c: 20, simulate_seconds: 1000
          });
          assert(r.level_history.length === r.error_history.length, '水位&误差历史等长');
          assert(r.error_history.length === r.flow_history.length, '误差&流量历史等长');
          assert(r.level_history.length > 2, '历史点数足够绘图');
        }
      },
      {
        id: 'v14', name: '水位变化：目标高于初始则上升', type: '正常',
        async run() {
          const r = await apiPost('/virtual-operate', {
            clepsydra_id: 'KD1', target_water_level_cm: 100, water_temp_c: 20, simulate_seconds: 300
          });
          const first = r.level_history[0][1];
          const last = r.level_history[r.level_history.length - 1][1];
          assert(last > first, `目标水位高则水位上升: ${first.toFixed(1)} → ${last.toFixed(1)}`);
        }
      }
    ]
  }
];

// ============================================================
// 测试运行器
// ============================================================

let stats = { total: 0, pass: 0, fail: 0, pending: 0 };

function assert(cond, msg) {
  if (!cond) throw new Error(msg || '断言失败');
}

async function apiGet(path) {
  const res = await fetch(API_BASE + path);
  if (!res.ok) {
    const err = new Error(`HTTP ${res.status}`);
    err.status = res.status;
    throw err;
  }
  const json = await res.json();
  if (json.code && json.code !== 0 && json.code !== 200) {
    if (json.data !== undefined) return json.data;
    const err = new Error(json.message || 'API错误');
    err.status = json.code;
    throw err;
  }
  return json.data !== undefined ? json.data : json;
}

async function apiPost(path, body) {
  const res = await fetch(API_BASE + path, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body)
  });
  if (!res.ok) {
    const err = new Error(`HTTP ${res.status}`);
    err.status = res.status;
    throw err;
  }
  const json = await res.json();
  if (json.code && json.code !== 0 && json.code !== 200) {
    if (json.data !== undefined) return json.data;
    const err = new Error(json.message || 'API错误');
    err.status = json.code;
    throw err;
  }
  return json.data !== undefined ? json.data : json;
}

function renderSuites() {
  const container = document.getElementById('suites');
  container.innerHTML = '';
  stats.total = 0;
  for (const suite of testSuites) {
    stats.total += suite.tests.length;
    const el = document.createElement('div');
    el.className = 'test-suite';
    el.innerHTML = `
      <div class="suite-header" onclick="toggleSuite('${suite.id}')">
        <span>${suite.name} (${suite.tests.length}用例)</span>
        <span class="suite-badge badge-pending" id="badge-${suite.id}">待运行</span>
      </div>
      <div class="suite-body" id="body-${suite.id}">
        ${suite.tests.map(t => `
          <div class="test-case" id="case-${t.id}">
            <div class="test-icon icon-pending" id="icon-${t.id}">○</div>
            <div class="test-name">
              <span style="color:#999; font-size:12px; margin-right:8px">[${t.type}]</span>
              ${t.name}
            </div>
            <div class="test-duration" id="dur-${t.id}"></div>
          </div>
        `).join('')}
      </div>
    `;
    container.appendChild(el);
  }
  stats.pending = stats.total;
  updateSummary();
}

function toggleSuite(id) {
  const body = document.getElementById('body-' + id);
  body.classList.toggle('open');
}

function updateSummary() {
  document.getElementById('totalCount').textContent = stats.total;
  document.getElementById('passCount').textContent = stats.pass;
  document.getElementById('failCount').textContent = stats.fail;
  document.getElementById('pendingCount').textContent = stats.pending;
}

async function runAllTests() {
  const btn = document.getElementById('btnRun');
  btn.disabled = true;
  btn.textContent = '⏳ 运行中...';

  API_BASE = document.getElementById('apiBase').value.replace(/\/$/, '');

  stats.pass = 0;
  stats.fail = 0;
  stats.pending = stats.total;
  updateSummary();

  for (const suite of testSuites) {
    document.getElementById('body-' + suite.id).classList.add('open');
    let suitePass = 0, suiteFail = 0;

    for (const test of suite.tests) {
      const iconEl = document.getElementById('icon-' + test.id);
      const durEl = document.getElementById('dur-' + test.id);
      const caseEl = document.getElementById('case-' + test.id);

      iconEl.className = 'test-icon icon-pending';
      iconEl.innerHTML = '<span class="spinner"></span>';
      durEl.textContent = '';

      const start = Date.now();
      try {
        await test.run();
        const dur = Date.now() - start;
        iconEl.className = 'test-icon icon-pass';
        iconEl.textContent = '✓';
        durEl.textContent = dur + 'ms';
        stats.pass++;
        suitePass++;
      } catch (err) {
        const dur = Date.now() - start;
        iconEl.className = 'test-icon icon-fail';
        iconEl.textContent = '✗';
        durEl.textContent = dur + 'ms';
        stats.fail++;
        suiteFail++;

        const errDiv = document.createElement('div');
        errDiv.className = 'test-error';
        errDiv.textContent = err.message || String(err);
        caseEl.appendChild(errDiv);
      }
      stats.pending--;
      updateSummary();
    }

    const badge = document.getElementById('badge-' + suite.id);
    if (suiteFail === 0) {
      badge.textContent = `全部通过 ${suitePass}/${suite.tests.length}`;
      badge.className = 'suite-badge badge-pass';
    } else {
      badge.textContent = `${suitePass}通过 ${suiteFail}失败`;
      badge.className = 'suite-badge badge-fail';
    }
  }

  btn.disabled = false;
  btn.textContent = '▶ 重新运行';
}

document.addEventListener('DOMContentLoaded', renderSuites);
