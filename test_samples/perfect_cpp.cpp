// Perfectly clean C++ file for certification
// Modern C++ with RAII and smart pointers

#include <iostream>
#include <vector>
#include <string>
#include <memory>
#include <optional>
#include <algorithm>
#include <numeric>

namespace synward {
namespace clean {

/// A simple counter with proper encapsulation
class Counter {
public:
    Counter() : count_(0) {}
    explicit Counter(uint64_t initial) : count_(initial) {}

    uint64_t increment() {
        return ++count_;
    }

    uint64_t get() const {
        return count_;
    }

    void reset() {
        count_ = 0;
    }

private:
    uint64_t count_;
};

/// Calculate factorial iteratively
uint64_t factorial(uint64_t n) {
    uint64_t result = 1;
    for (uint64_t i = 2; i <= n; ++i) {
        result *= i;
    }
    return result;
}

/// Generate Fibonacci sequence
std::vector<uint64_t> fibonacci(size_t n) {
    std::vector<uint64_t> seq;
    seq.reserve(n);

    uint64_t a = 0;
    uint64_t b = 1;

    for (size_t i = 0; i < n; ++i) {
        seq.push_back(a);
        uint64_t temp = a;
        a = b;
        b = temp + b;
    }

    return seq;
}

/// A point in 2D space
struct Point {
    double x;
    double y;

    static Point origin() {
        return {0.0, 0.0};
    }

    double distanceFromOrigin() const {
        return std::sqrt(x * x + y * y);
    }

    double distanceTo(const Point& other) const {
        double dx = x - other.x;
        double dy = y - other.y;
        return std::sqrt(dx * dx + dy * dy);
    }
};

/// A rectangle defined by two corners
class Rectangle {
public:
    Rectangle(Point topLeft, Point bottomRight)
        : topLeft_(topLeft), bottomRight_(bottomRight) {}

    double area() const {
        double width = std::abs(bottomRight_.x - topLeft_.x);
        double height = std::abs(topLeft_.y - bottomRight_.y);
        return width * height;
    }

    bool contains(const Point& point) const {
        return point.x >= topLeft_.x
            && point.x <= bottomRight_.x
            && point.y <= topLeft_.y
            && point.y >= bottomRight_.y;
    }

private:
    Point topLeft_;
    Point bottomRight_;
};

/// Status enum for processing state
enum class Status {
    Pending,
    Running,
    Completed,
    Failed
};

/// Check if status is terminal
inline bool isTerminal(Status status) {
    return status == Status::Completed || status == Status::Failed;
}

/// Check if status is active
inline bool isActive(Status status) {
    return status == Status::Pending || status == Status::Running;
}

/// Configuration for a service
class Config {
public:
    Config(std::string name, uint16_t port)
        : name_(std::move(name)), port_(port), debug_(false) {}

    Config& withDebug(bool debug = true) {
        debug_ = debug;
        return *this;
    }

    const std::string& name() const { return name_; }
    uint16_t port() const { return port_; }
    bool isDebug() const { return debug_; }

private:
    std::string name_;
    uint16_t port_;
    bool debug_;
};

/// A trait-like template for processors
template<typename T>
class Processor {
public:
    virtual ~Processor() = default;
    virtual std::string process(const T& input) = 0;
    virtual bool validate(const T& input) const = 0;
};

/// A simple string processor implementation
class StringProcessor : public Processor<std::string> {
public:
    explicit StringProcessor(std::string prefix)
        : prefix_(std::move(prefix)) {}

    std::string process(const std::string& input) override {
        return prefix_ + ": " + input;
    }

    bool validate(const std::string& input) const override {
        return !input.empty();
    }

private:
    std::string prefix_;
};

/// Sum all numbers in a container
template<typename Container>
auto sum(const Container& numbers) {
    return std::accumulate(numbers.begin(), numbers.end(), typename Container::value_type{});
}

/// Find the maximum value in a container
template<typename Container>
std::optional<typename Container::value_type> maxValue(const Container& numbers) {
    if (numbers.empty()) {
        return std::nullopt;
    }
    return *std::max_element(numbers.begin(), numbers.end());
}

/// Find the minimum value in a container
template<typename Container>
std::optional<typename Container::value_type> minValue(const Container& numbers) {
    if (numbers.empty()) {
        return std::nullopt;
    }
    return *std::min_element(numbers.begin(), numbers.end());
}

/// Filter positive numbers
std::vector<int32_t> filterPositive(const std::vector<int32_t>& numbers) {
    std::vector<int32_t> result;
    std::copy_if(numbers.begin(), numbers.end(), std::back_inserter(result),
                 [](int32_t x) { return x > 0; });
    return result;
}

/// Double all numbers
std::vector<int32_t> doubleAll(const std::vector<int32_t>& numbers) {
    std::vector<int32_t> result;
    result.reserve(numbers.size());
    std::transform(numbers.begin(), numbers.end(), std::back_inserter(result),
                   [](int32_t x) { return x * 2; });
    return result;
}

/// Resource management with RAII
class FileHandle {
public:
    explicit FileHandle(const std::string& filename)
        : file_(std::fopen(filename.c_str(), "r")) {
        if (!file_) {
            throw std::runtime_error("Cannot open file: " + filename);
        }
    }

    ~FileHandle() {
        if (file_) {
            std::fclose(file_);
        }
    }

    FileHandle(const FileHandle&) = delete;
    FileHandle& operator=(const FileHandle&) = delete;

    FileHandle(FileHandle&& other) noexcept : file_(other.file_) {
        other.file_ = nullptr;
    }

    FileHandle& operator=(FileHandle&& other) noexcept {
        if (this != &other) {
            if (file_) {
                std::fclose(file_);
            }
            file_ = other.file_;
            other.file_ = nullptr;
        }
        return *this;
    }

    std::FILE* get() const { return file_; }

private:
    std::FILE* file_;
};

} // namespace clean
} // namespace synward

int main() {
    using namespace synward::clean;

    // Test counter
    Counter counter;
    std::cout << "Counter: " << counter.get() << std::endl;
    counter.increment();
    std::cout << "After increment: " << counter.get() << std::endl;

    // Test factorial
    std::cout << "Factorial(5) = " << factorial(5) << std::endl;

    // Test fibonacci
    auto fib = fibonacci(10);
    std::cout << "Fibonacci(10): ";
    for (auto n : fib) {
        std::cout << n << " ";
    }
    std::cout << std::endl;

    // Test point
    Point p1{3.0, 4.0};
    std::cout << "Distance from origin: " << p1.distanceFromOrigin() << std::endl;

    // Test rectangle
    Rectangle rect(Point{0.0, 10.0}, Point{10.0, 0.0});
    std::cout << "Area: " << rect.area() << std::endl;

    // Test string processor
    StringProcessor proc("PREFIX");
    std::cout << "Processed: " << proc.process("hello") << std::endl;

    // Test utils
    std::vector<int32_t> numbers = {1, -2, 3, -4, 5};
    std::cout << "Sum: " << sum(numbers) << std::endl;
    std::cout << "Max: " << maxValue(numbers).value() << std::endl;
    std::cout << "Min: " << minValue(numbers).value() << std::endl;

    return 0;
}
