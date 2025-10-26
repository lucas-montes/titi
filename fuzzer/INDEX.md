# Elerem Fuzzer - Documentation Index

Complete documentation for the Elerem API testing and fuzzing framework.

## 📖 Quick Start

**New to the project?** Start here:
1. Read [README.md](README.md) - Architecture overview and examples
2. Review [SUMMARY.md](SUMMARY.md) - Executive summary
3. Check [EXAMPLES.md](EXAMPLES.md) - Practical usage patterns

## 📁 Documentation Files

### User-Facing Documentation

#### [README.md](README.md)
**Purpose:** Main documentation for users and contributors
**Contains:**
- Architecture overview with ASCII diagrams
- Component descriptions (Config, Planner, Scheduler, Executor, VUser)
- Data flow diagrams
- Metrics architecture
- Validation system
- Example usage code
- Performance characteristics
- Comparison with K6

**Audience:** Everyone - start here!

#### [EXAMPLES.md](EXAMPLES.md)
**Purpose:** Practical usage examples
**Contains:**
- Basic load test
- Ramping load test
- Stress test with error tolerance
- API testing with multiple endpoints
- Custom planner implementation
- Adaptive planner with feedback
- Metrics streaming examples
- Database recording examples
- Configuration examples (JSON, YAML)
- Website integration examples

**Audience:** Developers implementing tests

### Technical Documentation

#### [FLOW.md](FLOW.md)
**Purpose:** Detailed internal flows and lifecycle
**Contains:**
- Complete system flow diagram
- Metrics flow detail (3-level hierarchy)
- Validation flow
- Feedback loop (adaptive planning)
- Lifecycle stages (startup, main loop, shutdown)
- Concurrency model
- Error handling strategy
- Performance optimizations

**Audience:** Contributors, architects, advanced users

#### [TRAITS.md](TRAITS.md)
**Purpose:** Trait hierarchy and relationships
**Contains:**
- Core traits (TestPlanner)
- Key structs (Case, RequestTemplate, ExecutorType)
- Enums (Assertion, StoppingCondition)
- Metrics types (VUserMetrics, ExecutorSnapshot, GlobalSnapshot)
- Component relationships
- Ownership and lifetimes
- Channel communication
- Type safety guarantees

**Audience:** Contributors, Rust developers

#### [IMPLEMENTATION.md](IMPLEMENTATION.md)
**Purpose:** Implementation details and roadmap
**Contains:**
- Files created/updated
- Architecture validation checklist
- Data structures with code
- Metrics types detailed
- Next steps (5 phases)
- Integration points
- Testing strategy
- Performance goals
- Comparison with K6

**Audience:** Contributors, project maintainers

### Summary Documents

#### [SUMMARY.md](SUMMARY.md)
**Purpose:** Executive summary and quick reference
**Contains:**
- Complete architecture diagram
- Metrics flow
- Feedback loop
- Key design decisions
- Core data types
- Performance targets
- Next steps roadmap
- Comparison table

**Audience:** Everyone - quick overview

## 🗂️ Documentation by Topic

### Architecture

**High-level overview:**
- [README.md](README.md) - Architecture Overview
- [SUMMARY.md](SUMMARY.md) - Architecture Goals Achieved

**Detailed flows:**
- [FLOW.md](FLOW.md) - Complete System Flow
- [FLOW.md](FLOW.md) - Metrics Flow Detail
- [FLOW.md](FLOW.md) - Validation Flow
- [FLOW.md](FLOW.md) - Concurrency Model

**Design decisions:**
- [SUMMARY.md](SUMMARY.md) - Key Design Decisions
- [README.md](README.md) - Key Design Decisions
- [TRAITS.md](TRAITS.md) - Ownership and Lifetimes

### Implementation

**Core components:**
- [IMPLEMENTATION.md](IMPLEMENTATION.md) - Files Created/Updated
- [TRAITS.md](TRAITS.md) - Core Traits
- [TRAITS.md](TRAITS.md) - Key Structs

**Data structures:**
- [TRAITS.md](TRAITS.md) - Metrics Types
- [IMPLEMENTATION.md](IMPLEMENTATION.md) - Data Structures
- [SUMMARY.md](SUMMARY.md) - Core Data Types

**Next steps:**
- [IMPLEMENTATION.md](IMPLEMENTATION.md) - Next Steps (5 Phases)
- [SUMMARY.md](SUMMARY.md) - Next Steps Roadmap

### Usage

**Getting started:**
- [README.md](README.md) - Example Usage
- [EXAMPLES.md](EXAMPLES.md) - Basic Load Test
- [EXAMPLES.md](EXAMPLES.md) - Ramping Load Test

**Advanced usage:**
- [EXAMPLES.md](EXAMPLES.md) - Custom Planner Implementation
- [EXAMPLES.md](EXAMPLES.md) - Adaptive Planner with Feedback
- [EXAMPLES.md](EXAMPLES.md) - Metrics Streaming

**Integration:**
- [EXAMPLES.md](EXAMPLES.md) - Integration with Website
- [IMPLEMENTATION.md](IMPLEMENTATION.md) - Integration Points

### Metrics

**Architecture:**
- [README.md](README.md) - Metrics Architecture
- [SUMMARY.md](SUMMARY.md) - Metrics Flow
- [FLOW.md](FLOW.md) - Metrics Flow Detail

**Types:**
- [TRAITS.md](TRAITS.md) - Metrics Types
- [IMPLEMENTATION.md](IMPLEMENTATION.md) - Metrics Types

**Usage:**
- [EXAMPLES.md](EXAMPLES.md) - Metrics Streaming to Website
- [EXAMPLES.md](EXAMPLES.md) - Metrics Recording to Database

### Validation

**System:**
- [README.md](README.md) - Validation System
- [FLOW.md](FLOW.md) - Validation Flow
- [TRAITS.md](TRAITS.md) - Assertion and StoppingCondition

**Usage:**
- [EXAMPLES.md](EXAMPLES.md) - Stress Test with Error Tolerance

### Performance

**Characteristics:**
- [README.md](README.md) - Performance Characteristics
- [SUMMARY.md](SUMMARY.md) - Performance Characteristics
- [IMPLEMENTATION.md](IMPLEMENTATION.md) - Performance Characteristics

**Optimizations:**
- [FLOW.md](FLOW.md) - Performance Optimizations
- [IMPLEMENTATION.md](IMPLEMENTATION.md) - Performance Goals

## 🎯 Documentation by Role

### For Users

**I want to understand what this is:**
1. [SUMMARY.md](SUMMARY.md) - Quick overview
2. [README.md](README.md) - Full architecture guide

**I want to run tests:**
1. [EXAMPLES.md](EXAMPLES.md) - Basic Load Test
2. [EXAMPLES.md](EXAMPLES.md) - Configuration Examples
3. [EXAMPLES.md](EXAMPLES.md) - Running Tests

**I want advanced features:**
1. [EXAMPLES.md](EXAMPLES.md) - Custom Planner Implementation
2. [EXAMPLES.md](EXAMPLES.md) - Adaptive Planner with Feedback

### For Contributors

**I want to understand the architecture:**
1. [README.md](README.md) - Architecture Overview
2. [FLOW.md](FLOW.md) - Complete System Flow
3. [TRAITS.md](TRAITS.md) - Trait Hierarchy

**I want to implement features:**
1. [IMPLEMENTATION.md](IMPLEMENTATION.md) - Next Steps
2. [TRAITS.md](TRAITS.md) - Core Traits
3. [FLOW.md](FLOW.md) - Detailed Flows

**I want to understand the code:**
1. [TRAITS.md](TRAITS.md) - Component Relationships
2. [FLOW.md](FLOW.md) - Concurrency Model
3. [IMPLEMENTATION.md](IMPLEMENTATION.md) - Data Structures

### For Architects

**I want to evaluate the design:**
1. [SUMMARY.md](SUMMARY.md) - Architecture Goals Achieved
2. [README.md](README.md) - Key Design Decisions
3. [FLOW.md](FLOW.md) - Complete System Flow

**I want to understand tradeoffs:**
1. [IMPLEMENTATION.md](IMPLEMENTATION.md) - Design Tradeoffs
2. [SUMMARY.md](SUMMARY.md) - Key Learnings
3. [README.md](README.md) - Comparison with K6

## 📊 Visual Guides

### Diagrams in Documentation

**Architecture diagrams:**
- [README.md](README.md) - 5-Layer Architecture
- [SUMMARY.md](SUMMARY.md) - Architecture Overview
- [FLOW.md](FLOW.md) - Complete System Flow

**Flow diagrams:**
- [README.md](README.md) - Forward Flow
- [README.md](README.md) - Metrics Flow
- [README.md](README.md) - Feedback Loop
- [FLOW.md](FLOW.md) - Metrics Flow Detail
- [FLOW.md](FLOW.md) - Validation Flow

**Lifecycle diagrams:**
- [FLOW.md](FLOW.md) - Startup
- [FLOW.md](FLOW.md) - Main Loop
- [FLOW.md](FLOW.md) - Shutdown

**Concurrency diagrams:**
- [FLOW.md](FLOW.md) - Concurrency Model

## 🔍 Finding Information

### Quick Reference

**"How do I...?"**
- Run a basic test → [EXAMPLES.md](EXAMPLES.md) - Basic Load Test
- Create a ramping test → [EXAMPLES.md](EXAMPLES.md) - Ramping Load Test
- Implement a custom planner → [EXAMPLES.md](EXAMPLES.md) - Custom Planner Implementation
- Stream metrics to website → [EXAMPLES.md](EXAMPLES.md) - Metrics Streaming
- Configure tests → [EXAMPLES.md](EXAMPLES.md) - Configuration Examples

**"What is...?"**
- The architecture → [README.md](README.md) - Architecture Overview
- A planner → [README.md](README.md) - 2. Planner
- An executor → [README.md](README.md) - 4. Executor
- A virtual user → [README.md](README.md) - 5. Virtual Users
- MetricsHub → [README.md](README.md) - MetricsHub

**"Why does it...?"**
- Use Iterator pattern → [SUMMARY.md](SUMMARY.md) - Iterator-Based Planning
- Have share-nothing VUsers → [SUMMARY.md](SUMMARY.md) - Share-Nothing VUsers
- Use three-level metrics → [SUMMARY.md](SUMMARY.md) - Three-Level Metrics
- Separate stream and recorder → [README.md](README.md) - MetricsHub

### Search Tips

**By keyword:**
- `Iterator` → [TRAITS.md](TRAITS.md), [SUMMARY.md](SUMMARY.md)
- `Assertion` → [TRAITS.md](TRAITS.md), [README.md](README.md)
- `MetricsHub` → [README.md](README.md), [FLOW.md](FLOW.md)
- `Feedback` → [FLOW.md](FLOW.md), [EXAMPLES.md](EXAMPLES.md)
- `Channel` → [TRAITS.md](TRAITS.md), [FLOW.md](FLOW.md)

## 📚 Complete File List

### Core Documentation
- [README.md](README.md) - Main documentation (comprehensive guide)
- [SUMMARY.md](SUMMARY.md) - Executive summary (quick overview)
- [INDEX.md](INDEX.md) - This file (documentation index)

### Technical Documentation
- [FLOW.md](FLOW.md) - Detailed flows and lifecycle
- [TRAITS.md](TRAITS.md) - Trait hierarchy and relationships
- [IMPLEMENTATION.md](IMPLEMENTATION.md) - Implementation details and roadmap

### Usage Documentation
- [EXAMPLES.md](EXAMPLES.md) - Practical usage examples

### Source Code Documentation
- See `src/` directory for inline documentation in:
  - `planner.rs` - Test case generation
  - `scheduler.rs` - Test orchestration
  - `executor.rs` - Case execution
  - `vuser.rs` - Virtual user implementation
  - `config.rs` - Configuration types

## 🚀 Getting Started Path

**Recommended reading order:**

1. **Day 1 - Understanding**
   - [SUMMARY.md](SUMMARY.md) - 5 min read
   - [README.md](README.md) - 15 min read

2. **Day 2 - Usage**
   - [EXAMPLES.md](EXAMPLES.md) - Basic examples
   - Run first test
   - [EXAMPLES.md](EXAMPLES.md) - Advanced examples

3. **Day 3 - Deep Dive**
   - [FLOW.md](FLOW.md) - System flows
   - [TRAITS.md](TRAITS.md) - Code structure
   - [IMPLEMENTATION.md](IMPLEMENTATION.md) - Implementation details

4. **Day 4+ - Contributing**
   - Pick a task from [IMPLEMENTATION.md](IMPLEMENTATION.md) - Next Steps
   - Review related code in `src/`
   - Implement and test

## 📝 Documentation Standards

All documentation follows these principles:

1. **Clarity**: Simple language, clear examples
2. **Completeness**: Cover all aspects of the system
3. **Consistency**: Same terminology throughout
4. **Code Examples**: Rust code with comments
5. **Diagrams**: ASCII art for portability
6. **Cross-references**: Links between related topics

## 🔗 External Resources

**Rust documentation:**
- [The Rust Book](https://doc.rust-lang.org/book/)
- [Async Rust](https://rust-lang.github.io/async-book/)
- [tokio docs](https://tokio.rs)

**Testing references:**
- [K6 documentation](https://k6.io/docs/)
- [IPOG algorithm paper](https://csrc.nist.gov/publications/detail/conference-paper/2006/08/01/ipog-a-general-strategy-for-t-way-software-testing)

**Related projects:**
- [reqwest](https://docs.rs/reqwest/) - HTTP client
- [tokio-tungstenite](https://docs.rs/tokio-tungstenite/) - WebSocket

## 🎯 Conclusion

This documentation provides complete coverage of:
- ✅ Architecture and design
- ✅ Implementation details
- ✅ Usage examples
- ✅ Integration guides
- ✅ Performance characteristics
- ✅ Next steps and roadmap

**Start with [README.md](README.md) if new to the project!**

---

**Last updated:** 2024 (based on complete architecture implementation)
