use std::{
  sync::atomic::{AtomicU64, Ordering},
  time::Instant,
};

const METRIC_COUNT: usize = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
pub enum MetricId {
  // Arena Operations (0-14)
  ArenaNew = 0,
  ArenaAllocate = 1,
  ArenaAllocateSuccess = 2,
  ArenaAllocateFail = 3,
  ArenaDeallocate = 4,
  ArenaDeallocateSuccess = 5,
  ArenaDeallocateFail = 6,
  ArenaHasSpace = 7,
  ArenaHasSpaceTrue = 8,
  ArenaHasSpaceFalse = 9,
  ArenaCacheHit = 10,
  ArenaCacheMiss = 11,
  ArenaSegmentActivation = 12,
  ArenaSegmentDeactivation = 13,
  ArenaBitmapOperations = 14,

  // Segment Operations (15-34)
  SegmentNew = 15,
  SegmentNewSuccess = 16,
  SegmentNewFail = 17,
  SegmentAlloc = 18,
  SegmentAllocSuccess = 19,
  SegmentAllocFail = 20,
  SegmentDealloc = 21,
  SegmentDeallocSuccess = 22,
  SegmentDeallocFail = 23,
  SegmentStateTransitionFreeToPartial = 24,
  SegmentStateTransitionPartialToFull = 25,
  SegmentStateTransitionFullToPartial = 26,
  SegmentStateTransitionPartialToFree = 27,
  SegmentCacheHit = 28,
  SegmentCacheMiss = 29,
  SegmentPtrLookup = 30,
  SegmentPtrLookupSuccess = 31,
  SegmentPtrLookupFail = 32,
  SegmentBitmapSet = 33,
  SegmentBitmapClear = 34,

  // Queue Operations (35-49)
  QueueAllocate = 35,
  QueueAllocateSuccess = 36,
  QueueAllocateFail = 37,
  QueueDeallocate = 38,
  QueueDeallocateSuccess = 39,
  QueueDeallocateFail = 40,
  QueueSegmentDisplace = 41,
  QueueTrimFreeSegments = 42,
  QueueTrimSegmentsRemoved = 43,
  QueueGetAvailable = 44,
  QueueGetAvailableFromFree = 45,
  QueueGetAvailableFromPartial = 46,
  QueueGetAvailableNone = 47,
  QueueAddSegment = 48,
  QueueNewSegmentCreated = 49,

  // Heap Operations (50-69)
  HeapAllocate = 50,
  HeapAllocateSuccess = 51,
  HeapAllocateFail = 52,
  HeapDeallocate = 53,
  HeapDeallocateSuccess = 54,
  HeapDeallocateFail = 55,
  HeapAllocSmall = 56,
  HeapAllocLarge = 57,
  HeapDeallocSmall = 58,
  HeapDeallocLarge = 59,
  HeapRemoteProcessing = 60,
  HeapRemoteBatched = 61,
  HeapRemoteSkipped = 62,
  HeapOperationsCounter = 63,
  HeapClassLookup = 64,
  HeapClassLookupSuccess = 65,
  HeapClassLookupFail = 66,
  HeapInvalidSize = 67,
  HeapInvalidPointer = 68,
  HeapThreadMismatch = 69,

  // Static Management (70-79)
  StaticCreateArena = 70,
  StaticCreateArenaSuccess = 71,
  StaticCreateArenaFail = 72,
  StaticAddArena = 73,
  StaticAddArenaSuccess = 74,
  StaticAddArenaFail = 75,
  StaticSegmentLookup = 76,
  StaticSegmentLookupSuccess = 77,
  StaticSegmentLookupFail = 78,
  StaticArenaGrowth = 79,

  // Size Class Analysis (80-94)
  SizeClassSmall = 80,
  SizeClassMedium = 81,
  SizeClassLarge = 82,
  SizeClassHuge = 83,
  SizeClassPerfectFit = 84,
  SizeClassWaste = 85,
  SizeClass0To7 = 86,
  SizeClass8To15 = 87,
  SizeClass16To31 = 88,
  SizeClass32To47 = 89,
  SizeClass48To63 = 90,
  SizeClass64To79 = 91,
  SizeClass80Plus = 92,
  SizeClassUtilizationHigh = 93,
  SizeClassUtilizationLow = 94,

  // Error Conditions (95-99)
  ErrorInsufficientMemory = 95,
  ErrorInvalidPointer = 96,
  ErrorBitmapError = 97,
  ErrorSegmentError = 98,
  ErrorMapError = 99,
}

impl MetricId {
  pub const fn name(self) -> &'static str {
    match self {
      MetricId::ArenaNew => "arena_new",
      MetricId::ArenaAllocate => "arena_allocate",
      MetricId::ArenaAllocateSuccess => "arena_allocate_success",
      MetricId::ArenaAllocateFail => "arena_allocate_fail",
      MetricId::ArenaDeallocate => "arena_deallocate",
      MetricId::ArenaDeallocateSuccess => "arena_deallocate_success",
      MetricId::ArenaDeallocateFail => "arena_deallocate_fail",
      MetricId::ArenaHasSpace => "arena_has_space",
      MetricId::ArenaHasSpaceTrue => "arena_has_space_true",
      MetricId::ArenaHasSpaceFalse => "arena_has_space_false",
      MetricId::ArenaCacheHit => "arena_cache_hit",
      MetricId::ArenaCacheMiss => "arena_cache_miss",
      MetricId::ArenaSegmentActivation => "arena_segment_activation",
      MetricId::ArenaSegmentDeactivation => "arena_segment_deactivation",
      MetricId::ArenaBitmapOperations => "arena_bitmap_operations",

      MetricId::SegmentNew => "segment_new",
      MetricId::SegmentNewSuccess => "segment_new_success",
      MetricId::SegmentNewFail => "segment_new_fail",
      MetricId::SegmentAlloc => "segment_alloc",
      MetricId::SegmentAllocSuccess => "segment_alloc_success",
      MetricId::SegmentAllocFail => "segment_alloc_fail",
      MetricId::SegmentDealloc => "segment_dealloc",
      MetricId::SegmentDeallocSuccess => "segment_dealloc_success",
      MetricId::SegmentDeallocFail => "segment_dealloc_fail",
      MetricId::SegmentStateTransitionFreeToPartial => "segment_free_to_partial",
      MetricId::SegmentStateTransitionPartialToFull => "segment_partial_to_full",
      MetricId::SegmentStateTransitionFullToPartial => "segment_full_to_partial",
      MetricId::SegmentStateTransitionPartialToFree => "segment_partial_to_free",
      MetricId::SegmentCacheHit => "segment_cache_hit",
      MetricId::SegmentCacheMiss => "segment_cache_miss",
      MetricId::SegmentPtrLookup => "segment_ptr_lookup",
      MetricId::SegmentPtrLookupSuccess => "segment_ptr_lookup_success",
      MetricId::SegmentPtrLookupFail => "segment_ptr_lookup_fail",
      MetricId::SegmentBitmapSet => "segment_bitmap_set",
      MetricId::SegmentBitmapClear => "segment_bitmap_clear",

      MetricId::QueueAllocate => "queue_allocate",
      MetricId::QueueAllocateSuccess => "queue_allocate_success",
      MetricId::QueueAllocateFail => "queue_allocate_fail",
      MetricId::QueueDeallocate => "queue_deallocate",
      MetricId::QueueDeallocateSuccess => "queue_deallocate_success",
      MetricId::QueueDeallocateFail => "queue_deallocate_fail",
      MetricId::QueueSegmentDisplace => "queue_segment_displace",
      MetricId::QueueTrimFreeSegments => "queue_trim_free_segments",
      MetricId::QueueTrimSegmentsRemoved => "queue_trim_segments_removed",
      MetricId::QueueGetAvailable => "queue_get_available",
      MetricId::QueueGetAvailableFromFree => "queue_get_available_from_free",
      MetricId::QueueGetAvailableFromPartial => "queue_get_available_from_partial",
      MetricId::QueueGetAvailableNone => "queue_get_available_none",
      MetricId::QueueAddSegment => "queue_add_segment",
      MetricId::QueueNewSegmentCreated => "queue_new_segment_created",

      MetricId::HeapAllocate => "heap_allocate",
      MetricId::HeapAllocateSuccess => "heap_allocate_success",
      MetricId::HeapAllocateFail => "heap_allocate_fail",
      MetricId::HeapDeallocate => "heap_deallocate",
      MetricId::HeapDeallocateSuccess => "heap_deallocate_success",
      MetricId::HeapDeallocateFail => "heap_deallocate_fail",
      MetricId::HeapAllocSmall => "heap_alloc_small",
      MetricId::HeapAllocLarge => "heap_alloc_large",
      MetricId::HeapDeallocSmall => "heap_dealloc_small",
      MetricId::HeapDeallocLarge => "heap_dealloc_large",
      MetricId::HeapRemoteProcessing => "heap_remote_processing",
      MetricId::HeapRemoteBatched => "heap_remote_batched",
      MetricId::HeapRemoteSkipped => "heap_remote_skipped",
      MetricId::HeapOperationsCounter => "heap_operations_counter",
      MetricId::HeapClassLookup => "heap_class_lookup",
      MetricId::HeapClassLookupSuccess => "heap_class_lookup_success",
      MetricId::HeapClassLookupFail => "heap_class_lookup_fail",
      MetricId::HeapInvalidSize => "heap_invalid_size",
      MetricId::HeapInvalidPointer => "heap_invalid_pointer",
      MetricId::HeapThreadMismatch => "heap_thread_mismatch",

      MetricId::StaticCreateArena => "static_create_arena",
      MetricId::StaticCreateArenaSuccess => "static_create_arena_success",
      MetricId::StaticCreateArenaFail => "static_create_arena_fail",
      MetricId::StaticAddArena => "static_add_arena",
      MetricId::StaticAddArenaSuccess => "static_add_arena_success",
      MetricId::StaticAddArenaFail => "static_add_arena_fail",
      MetricId::StaticSegmentLookup => "static_segment_lookup",
      MetricId::StaticSegmentLookupSuccess => "static_segment_lookup_success",
      MetricId::StaticSegmentLookupFail => "static_segment_lookup_fail",
      MetricId::StaticArenaGrowth => "static_arena_growth",

      MetricId::SizeClassSmall => "size_class_small",
      MetricId::SizeClassMedium => "size_class_medium",
      MetricId::SizeClassLarge => "size_class_large",
      MetricId::SizeClassHuge => "size_class_huge",
      MetricId::SizeClassPerfectFit => "size_class_perfect_fit",
      MetricId::SizeClassWaste => "size_class_waste",
      MetricId::SizeClass0To7 => "size_class_0_to_7",
      MetricId::SizeClass8To15 => "size_class_8_to_15",
      MetricId::SizeClass16To31 => "size_class_16_to_31",
      MetricId::SizeClass32To47 => "size_class_32_to_47",
      MetricId::SizeClass48To63 => "size_class_48_to_63",
      MetricId::SizeClass64To79 => "size_class_64_to_79",
      MetricId::SizeClass80Plus => "size_class_80_plus",
      MetricId::SizeClassUtilizationHigh => "size_class_utilization_high",
      MetricId::SizeClassUtilizationLow => "size_class_utilization_low",

      MetricId::ErrorInsufficientMemory => "error_insufficient_memory",
      MetricId::ErrorInvalidPointer => "error_invalid_pointer",
      MetricId::ErrorBitmapError => "error_bitmap_error",
      MetricId::ErrorSegmentError => "error_segment_error",
      MetricId::ErrorMapError => "error_map_error",
    }
  }

  pub const fn category(self) -> &'static str {
    match self as usize {
      0..=14 => "Arena",
      15..=34 => "Segment",
      35..=49 => "Queue",
      50..=69 => "Heap",
      70..=79 => "Static",
      80..=94 => "SizeClass",
      95..=99 => "Error",
      _ => "Unknown",
    }
  }
}

static METRICS: [AtomicU64; METRIC_COUNT] = [
  AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0),
  AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0),
  AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0),
  AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0),
  AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0),
  AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0),
  AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0),
  AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0),
  AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0),
  AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0),
  AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0),
  AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0),
  AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0),
  AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0),
  AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0),
  AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0),
  AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0),
  AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0),
  AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0),
  AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0),
];

static START_TIME: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();

#[macro_export]
macro_rules! metric {
  ($id:expr) => {
    $crate::metrics::record_metric($id)
  };
  ($id:expr, $count:expr) => {
    $crate::metrics::record_metric_count($id, $count)
  };
}

pub fn record_metric(id: MetricId) {
  record_metric_count(id, 1);
}

pub fn record_metric_count(id: MetricId, count: u64) {
  let index = id as usize;
  if index < METRIC_COUNT {
    METRICS[index].fetch_add(count, Ordering::Relaxed);
  }
}

pub fn get_metric(id: MetricId) -> u64 {
  let index = id as usize;
  if index < METRIC_COUNT {
    METRICS[index].load(Ordering::Relaxed)
  } else {
    0
  }
}

pub fn start_summary() {
  for metric in &METRICS {
    metric.store(0, Ordering::Relaxed);
  }
  START_TIME.set(Instant::now()).ok();
  println!("Metrics collection started. All counters reset to zero.");
}

pub fn print_summary() {
  let start_time = START_TIME.get().copied().unwrap_or_else(Instant::now);
  let elapsed = start_time.elapsed();

  println!("\n=== TinyAlloc Metrics Summary ===");
  println!("Collection period: {:.2}s", elapsed.as_secs_f64());
  println!();

  let categories = ["Arena", "Segment", "Queue", "Heap", "Static", "SizeClass", "Error"];
  let mut total_operations = 0u64;

  for category in categories {
    let mut category_total = 0u64;
    let mut category_metrics = Vec::new();

    for i in 0..METRIC_COUNT {
      let metric_id = unsafe { core::mem::transmute::<usize, MetricId>(i) };
      if metric_id.category() == category {
        let count = get_metric(metric_id);
        if count > 0 {
          category_metrics.push((metric_id, count));
          category_total += count;
        }
      }
    }

    if !category_metrics.is_empty() {
      println!("=== {} Operations ({} total) ===", category, category_total);
      for (metric_id, count) in category_metrics {
        let rate = if elapsed.as_secs_f64() > 0.0 {
          count as f64 / elapsed.as_secs_f64()
        } else {
          0.0
        };
        println!("  {:30} {:12} ({:8.1}/s)", metric_id.name(), count, rate);
      }
      println!();
      total_operations += category_total;
    }
  }

  if total_operations > 0 {
    let total_rate = if elapsed.as_secs_f64() > 0.0 {
      total_operations as f64 / elapsed.as_secs_f64()
    } else {
      0.0
    };
    println!("=== Summary ===");
    println!("Total operations: {} ({:.1}/s)", total_operations, total_rate);
    
    print_derived_metrics();
  } else {
    println!("No metrics recorded yet. Call start_summary() to begin collection.");
  }
}

fn print_derived_metrics() {
  println!("\n=== Derived Metrics ===");

  let arena_success_rate = calculate_success_rate(
    MetricId::ArenaAllocateSuccess,
    MetricId::ArenaAllocate,
  );
  let segment_success_rate = calculate_success_rate(
    MetricId::SegmentAllocSuccess,
    MetricId::SegmentAlloc,
  );
  let queue_success_rate = calculate_success_rate(
    MetricId::QueueAllocateSuccess,
    MetricId::QueueAllocate,
  );
  let heap_success_rate = calculate_success_rate(
    MetricId::HeapAllocateSuccess,
    MetricId::HeapAllocate,
  );

  if arena_success_rate >= 0.0 {
    println!("  Arena success rate:       {:.1}%", arena_success_rate * 100.0);
  }
  if segment_success_rate >= 0.0 {
    println!("  Segment success rate:     {:.1}%", segment_success_rate * 100.0);
  }
  if queue_success_rate >= 0.0 {
    println!("  Queue success rate:       {:.1}%", queue_success_rate * 100.0);
  }
  if heap_success_rate >= 0.0 {
    println!("  Heap success rate:        {:.1}%", heap_success_rate * 100.0);
  }

  let cache_hit_rate = calculate_hit_rate(
    MetricId::ArenaCacheHit,
    MetricId::ArenaCacheMiss,
  );
  let segment_cache_hit_rate = calculate_hit_rate(
    MetricId::SegmentCacheHit,
    MetricId::SegmentCacheMiss,
  );

  if cache_hit_rate >= 0.0 {
    println!("  Arena cache hit rate:     {:.1}%", cache_hit_rate * 100.0);
  }
  if segment_cache_hit_rate >= 0.0 {
    println!("  Segment cache hit rate:   {:.1}%", segment_cache_hit_rate * 100.0);
  }

  let small_vs_large = get_metric(MetricId::HeapAllocSmall) as f64 /
    (get_metric(MetricId::HeapAllocSmall) + get_metric(MetricId::HeapAllocLarge)) as f64;
  if small_vs_large.is_finite() && small_vs_large >= 0.0 {
    println!("  Small allocation ratio:   {:.1}%", small_vs_large * 100.0);
  }
}

fn calculate_success_rate(success_metric: MetricId, total_metric: MetricId) -> f64 {
  let success = get_metric(success_metric) as f64;
  let total = get_metric(total_metric) as f64;
  if total > 0.0 {
    success / total
  } else {
    -1.0
  }
}

fn calculate_hit_rate(hit_metric: MetricId, miss_metric: MetricId) -> f64 {
  let hits = get_metric(hit_metric) as f64;
  let misses = get_metric(miss_metric) as f64;
  let total = hits + misses;
  if total > 0.0 {
    hits / total
  } else {
    -1.0
  }
}