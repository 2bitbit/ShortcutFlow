"""HtmlParser — 解析上游 HTML 内容并提取结构化数据"""
import sys, json
from html.parser import HTMLParser

class SelectorParser(HTMLParser):
    """简易 CSS 选择器解析：支持 tag、.class、#id 的简单组合"""
    def __init__(self, selector, extract, attr=None):
        super().__init__()
        self.selector = self._parse_selector(selector)
        self.extract = extract
        self.attr = attr
        self.results = []
        self._path = []
        self._capture = False
        self._capture_depth = 0
        self._current_data = []
    
    def _parse_selector(self, sel):
        """解析简单选择器: tag.class#id"""
        parts = {"tag": None, "classes": set(), "id": None}
        current = ""
        current_type = "tag"
        for ch in sel.strip():
            if ch == ".":
                if current and current_type == "tag":
                    parts["tag"] = current.lower() if current else None
                elif current and current_type == "class":
                    parts["classes"].add(current.lower())
                current = ""
                current_type = "class"
            elif ch == "#":
                if current and current_type == "tag":
                    parts["tag"] = current.lower() if current else None
                elif current and current_type == "class":
                    parts["classes"].add(current.lower())
                current = ""
                current_type = "id"
            else:
                current += ch
        if current:
            if current_type == "tag":
                parts["tag"] = current.lower() if current else None
            elif current_type == "class":
                parts["classes"].add(current.lower())
            elif current_type == "id":
                parts["id"] = current
        return parts
    
    def _matches(self, tag, attrs):
        s = self.selector
        if s["tag"] and tag.lower() != s["tag"]:
            return False
        if s["classes"]:
            cls = dict(attrs).get("class", "").split()
            if not s["classes"].issubset(set(c.lower() for c in cls)):
                return False
        if s["id"] and dict(attrs).get("id") != s["id"]:
            return False
        return True
    
    def handle_starttag(self, tag, attrs):
        if self._capture:
            self._capture_depth += 1
            if self.extract == "html":
                self._current_data.append(self.get_starttag_text())
            return
        
        if self._matches(tag, attrs):
            self._capture = True
            self._capture_depth = 1
            if self.extract == "html":
                self._current_data.append(self.get_starttag_text())
            elif self.extract == "attr" and self.attr:
                val = dict(attrs).get(self.attr, "")
                if val:
                    self.results.append(val)
    
    def handle_endtag(self, tag):
        if self._capture:
            self._capture_depth -= 1
            if self.extract == "html":
                self._current_data.append(f"</{tag}>")
            if self._capture_depth == 0:
                self._capture = False
                if self.extract == "html":
                    self.results.append("".join(self._current_data))
                elif self.extract == "text":
                    self.results.append("".join(self._current_data).strip())
                self._current_data = []
    
    def handle_data(self, data):
        if self._capture and self.extract in ("text", "html"):
            self._current_data.append(data)


def main():
    raw = sys.stdin.read().strip()
    if not raw:
        print(json.dumps({"error": "无输入数据"}))
        return
    
    # Parse the DataEnvelope JSON from Shell
    try:
        envelope = json.loads(raw)
        html = envelope.get("payload", raw)
    except:
        html = raw
    
    # Read config from metadata
    selector = "body"
    extract = "text"
    attr = None
    try:
        meta = envelope.get("metadata", {})
        selector = meta.get("selector", selector)
        extract = meta.get("extract", extract)
        attr = meta.get("attr")
    except:
        pass
    
    parser = SelectorParser(selector, extract, attr)
    try:
        parser.feed(html)
    except Exception as e:
        print(json.dumps({"error": f"解析失败: {e}"}))
        return
    
    result = "\n".join(parser.results) if parser.results else ""
    print(result)


if __name__ == "__main__":
    main()
