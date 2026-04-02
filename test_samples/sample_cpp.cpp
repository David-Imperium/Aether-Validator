// Test file for Synward C++ validation
// Contains various patterns to test the validation layers

#include <iostream>
#include <vector>
#include <string>
#include <memory>
#include <optional>

namespace synward {
namespace test {

/// A simple class for testing
class User {
public:
    User(uint64_t id, std::string name, std::string email)
        : id_(id), name_(std::move(name)), email_(std::move(email)), active_(true) {}

    bool validateEmail() const {
        return email_.find('@') != std::string::npos && 
               email_.find('.') != std::string::npos;
    }

    void deactivate() { active_ = false; }

    // Getters
    uint64_t id() const { return id_; }
    const std::string& name() const { return name_; }
    const std::string& email() const { return email_; }
    bool isActive() const { return active_; }

private:
    uint64_t id_;
    std::string name_;
    std::string email_;
    bool active_;
};

/// Calculate factorial recursively
uint64_t factorial(uint64_t n) {
    if (n <= 1) return 1;
    return n * factorial(n - 1);
}

/// Process a list of items
std::vector<int32_t> processItems(const std::vector<int32_t>& items) {
    std::vector<int32_t> result;
    for (const auto& item : items) {
        if (item > 0) {
            result.push_back(item * 2);
        }
    }
    return result;
}

/// Problematic function - raw pointer without ownership
int* createArray(size_t size) {
    // Issue: raw new without corresponding delete
    return new int[size];
}

/// Problematic function - memory leak
void memoryLeakExample() {
    // Issue: memory leak - no delete
    int* data = new int[100];
    data[0] = 42;
    std::cout << data[0] << std::endl;
    // Missing: delete[] data;
}

/// Problematic function - use after free
void useAfterFreeExample() {
    int* ptr = new int(42);
    delete ptr;
    // Issue: use after free
    std::cout << *ptr << std::endl;
}

/// Problematic function - double free
void doubleFreeExample() {
    int* ptr = new int(42);
    delete ptr;
    // Issue: double free
    delete ptr;
}

/// Problematic function - null pointer dereference
void nullPointerExample(int* ptr) {
    // Issue: potential null dereference without check
    *ptr = 10;
}

/// Function with unreachable code
int unreachableExample(int x) {
    if (x > 0) {
        return x;
    } else {
        return -x;
    }
    // Unreachable code
    return x * 2;
}

/// Deep nesting example
int deepNestingExample(int a, int b, int c, int d, int e) {
    int total = 0;
    if (a > 0) {
        if (b > 0) {
            if (c > 0) {
                if (d > 0) {
                    if (e > 0) {
                        total = a + b + c + d + e;
                    }
                }
            }
        }
    }
    return total;
}

/// TODO comment for testing
void todoFunction() {
    // TODO: implement this function properly
    // FIXME: this is broken
    // XXX: hack alert
}

/// Proper RAII example
class ResourceManager {
public:
    ResourceManager() : data_(std::make_unique<int[]>(100)) {}
    
    int& operator[](size_t index) { return data_[index]; }
    const int& operator[](size_t index) const { return data_[index]; }

private:
    std::unique_ptr<int[]> data_;
};

/// Modern C++ example with smart pointers
class ModernUser {
public:
    static std::shared_ptr<ModernUser> create(uint64_t id, std::string name) {
        return std::shared_ptr<ModernUser>(new ModernUser(id, std::move(name)));
    }

    uint64_t id() const { return id_; }
    const std::string& name() const { return name_; }

private:
    ModernUser(uint64_t id, std::string name) 
        : id_(id), name_(std::move(name)) {}

    uint64_t id_;
    std::string name_;
};

/// Enum class for status
enum class Status {
    Pending,
    InProgress,
    Completed,
    Failed
};

/// Template function example
template<typename T>
T maxValue(T a, T b) {
    return (a > b) ? a : b;
}

/// Concept-like template (C++20 style comment)
template<typename T>
concept Numeric = std::is_arithmetic_v<T>;

/// Variadic template example
template<typename... Args>
void printAll(Args... args) {
    (std::cout << ... << args) << std::endl;
}

} // namespace test
} // namespace synward

int main() {
    using namespace synward::test;

    // Test factorial
    std::cout << "Factorial(5) = " << factorial(5) << std::endl;

    // Test User
    User user(1, "Test User", "test@example.com");
    std::cout << "User: " << user.name() << ", Email valid: " << user.validateEmail() << std::endl;

    // Test processItems
    std::vector<int32_t> items = {1, -2, 3, -4, 5};
    auto processed = processItems(items);
    std::cout << "Processed items: ";
    for (const auto& item : processed) {
        std::cout << item << " ";
    }
    std::cout << std::endl;

    // Test template
    std::cout << "Max(10, 20) = " << maxValue(10, 20) << std::endl;

    return 0;
}
