# Java源码到Class文件复制工具

这个工具用于将Java源文件对应的已编译class文件复制到指定目录，同时复制源目录中的非Java文件，保持原有的目录结构。

## 功能

- 根据提供的Java源文件，查找对应的class文件
- 支持内部类（如`MyClass$InnerClass.class`）
- 保持源文件的包结构
- 复制源目录中的非Java文件（如XML、配置文件等）
- 如果任何Java文件找不到对应的class文件，则中止操作并报错
- 详细打印每个复制文件的信息，包括源文件、class文件、文件大小和JDK版本
- 检测每个class文件的JDK版本，并在发现多个不同JDK版本时发出警告
- 提供源文件和class文件的汇总统计信息

## 使用方法

```bash
src_to_class -s <源代码目录> -c <class文件目录> -o <输出目录>
```

### 参数说明

- `-s, --source-dir`: Java源代码所在的目录
- `-c, --class-dir`: 编译后的class文件所在的目录
- `-o, --output-dir`: 要输出class文件的目标目录

### 示例

```bash
cargo run -- -s ./src/main/java -c ./target/classes -o ./output
```

## 输出格式

程序会首先复制非Java文件，然后复制Java文件对应的class文件：

```
开始复制非Java文件...
非Java文件：config/application.xml，大小：2048 字节
非Java文件：resources/config.properties，大小：1024 字节
----------------------------------------
开始复制Java文件对应的class文件并检查JDK版本...
----------------------------------------
源文件：com/example/Test.java，class文件：com/example/Test.class，大小：1024 字节，JDK版本：JDK 8
源文件：com/example/Test.java，class文件：com/example/Test$Inner.class，大小：512 字节，JDK版本：JDK 8
----------------------------------------
源文件：org/sample/Main.java，class文件：org/sample/Main.class，大小：2048 字节，JDK版本：JDK 11
----------------------------------------

--- 汇总信息 ---
源文件总数: 2
class文件总数: 3
非Java文件总数: 2
复制文件总计: 5

-- 不同JDK版本文件统计 --
JDK 8: 2 个文件
JDK 11: 1 个文件
```

## JDK版本检测

工具会读取每个class文件的文件头，确定其编译使用的JDK版本。支持检测以下JDK版本：

- JDK 1.1 - JDK 1.4
- JDK 5 - JDK 21

如果一批文件中包含不同JDK版本编译的class文件，工具会发出警告并显示每个版本对应的文件数量。

## 非Java文件复制

工具会自动复制源目录中的所有非Java文件到输出目录，包括但不限于：

- XML配置文件
- Properties文件
- 资源文件
- 其他配置文件

这样可以避免手动复制配置文件等非源码文件的麻烦。

## 注意事项

- 工具会递归查找源代码目录下的所有文件
- 对于.java文件，会查找对应的所有class文件（包括内部类）
- 对于非.java文件，直接从源目录复制到输出目录
- 如果有任何Java文件找不到对应的class文件，工具会报错并且不会复制任何文件
- 输出目录会自动创建（如果不存在）
- 检测到不同JDK版本的文件时，仍会继续复制，但会发出警告 