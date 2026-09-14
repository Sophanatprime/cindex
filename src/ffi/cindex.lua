local ffi = require("ffi");
local bit = require("bit");
local string_buffer = require("string.buffer");
local cindex_table  = require("cindex.table");
local icu_table     = require("icu.table");
local script = ...;

ffi.cdef[[
int printf(const char *fmt, ...);
int sprintf(char* buffer, const char* format, ...);

typedef struct { uint8_t* ptr; size_t len; } StrRef;
typedef struct { uint8_t* ptr; size_t len; } OrderedStrokes;
typedef struct { uint32_t inner; } Char;

typedef struct { void* inner; } IndexEntryArray;
typedef struct { void* inner; } IndexEntryRef;
typedef struct { void* inner; } IndexEntryRefMut;
typedef struct { void* inner; } MergedEntryRef;
typedef struct { void* inner; } Emoji;

typedef struct {
    uint8_t radical;
    int8_t additional_strokes;
    uint8_t k_total_strokes;
    uint8_t glyph_total_strokes;
    uint32_t _extra_data;
} CjkInfo;

typedef void pcre2_code_8;
typedef void pcre2_match_data_8;
typedef void pcre2_compile_context_8;
typedef void pcre2_general_context_8;
typedef void pcre2_match_context_8;
typedef struct {
    pcre2_code_8* code;
    pcre2_match_data_8* match_data;
} pcre2_regex_t;

typedef struct {
    bool (*idx_input_read_all)(const void*, void*, const StrRef*);
    bool (*ikv_input_read_all)(const void*, void*, const StrRef*);
    bool (*write_to_output)(void*, const uint8_t*, size_t);
    bool (*log)(uint8_t, const uint8_t*, size_t);
    void* ist_input;
    struct {
        bool (*keyword)(void*, StrRef*);
        uint32_t (*arg_open)(void*);
        uint32_t (*arg_close)(void*);
        uint32_t (*range_open)(void*);
        uint32_t (*range_close)(void*);
        uint32_t (*level)(void*);
        uint32_t (*actual)(void*);
        uint32_t (*encap)(void*);
        uint32_t (*quote)(void*);
        uint32_t (*escape)(void*);
        bool (*page_compositor)(void*, StrRef*);
        uint32_t (*comment)(void*);
        bool (*separator)(void*, StrRef*);
    } ist_input_fn;
    void* ist_output;
    struct {
        bool (*preamble)(void*, StrRef*);
        bool (*postamble)(void*, StrRef*);
        bool (*group_skip)(void*, StrRef*);
        bool (*heading_prefix)(void*, StrRef*);
        bool (*heading_suffix)(void*, StrRef*);
        int32_t (*headings_flag)(void*);
        bool (*numhead_positive)(void*, StrRef*);
        bool (*numhead_negative)(void*, StrRef*);
        bool (*symhead_positive)(void*, StrRef*);
        bool (*symhead_negative)(void*, StrRef*);
        bool (*item_0)(void*, StrRef*);
        bool (*item_1)(void*, StrRef*);
        bool (*item_2)(void*, StrRef*);
        bool (*item_01)(void*, StrRef*);
        bool (*item_x1)(void*, StrRef*);
        bool (*item_12)(void*, StrRef*);
        bool (*item_x2)(void*, StrRef*);
        bool (*delim_0)(void*, StrRef*);
        bool (*delim_1)(void*, StrRef*);
        bool (*delim_2)(void*, StrRef*);
        bool (*delim_n)(void*, StrRef*);
        bool (*delim_r)(void*, StrRef*);
        bool (*delim_t)(void*, StrRef*);
        bool (*encap_prefix)(void*, StrRef*);
        bool (*encap_infix)(void*, StrRef*);
        bool (*encap_suffix)(void*, StrRef*);
        bool (*page_precedence)(void*, StrRef*);
        bool (*suffix_2p)(void*, StrRef*);
        bool (*suffix_3p)(void*, StrRef*);
        bool (*suffix_mp)(void*, StrRef*);
        bool (*stroke_prefix)(void*, StrRef*);
        bool (*stroke_suffix)(void*, StrRef*);
        bool (*radical_prefix)(void*, StrRef*);
        bool (*radical_suffix)(void*, StrRef*);
        int32_t (*radical_simplified_flag)(void*);
        bool (*radical_simplified_prefix)(void*, StrRef*);
        bool (*radical_simplified_delimiter)(void*, StrRef*);
        bool (*radical_simplified_suffix)(void*, StrRef*);
    } ist_output_fn;
    struct {
        size_t (*strlen)(const uint8_t*);
        bool (*from_buf)(const uint8_t*, size_t, StrRef*);
        int64_t (*cmp)(const void*, const void*);
        int64_t (*cmp_buf)(void*, const uint8_t*, size_t);
        int64_t (*cmp_ignore_case)(const uint8_t*, size_t, const uint8_t*, size_t);
        bool (*slice)(const StrRef*, size_t, size_t, StrRef*);
        uint32_t (*head)(const uint8_t*);
        uint32_t (*char_next)(StrRef*);
        bool (*line_next)(StrRef*, StrRef*); // rest str, out str
        bool (*split_char_next)(StrRef*, uint32_t, StrRef*);
        bool (*split_str_next)(StrRef*, const uint8_t*, size_t, StrRef*);
        bool (*split_spaces_next)(StrRef*, StrRef*);
        bool (*graphemes_next)(StrRef*, StrRef*);
        bool (*unicode_words_next)(StrRef*, StrRef*);
        bool (*split_word_bounds_next)(StrRef*, StrRef*);
        bool (*trim_ascii_spaces)(const StrRef*, bool, bool, StrRef*);
    } strref;
    struct {
        uint32_t (*from_i64)(int64_t);
        bool (*is_valid)(uint32_t);
        bool (*encode_utf8)(uint32_t, uint8_t*);
        size_t (*len_utf8)(uint32_t);
        uint32_t (*script)(uint32_t);
        uint8_t (*general_category)(uint32_t);
        bool (*is_alphabetic)(uint32_t);
        bool (*is_numeric)(uint32_t);
        bool (*is_math)(uint32_t);
        bool (*is_ideographic)(uint32_t);
        bool (*_is_visible)(uint32_t);
        uint32_t (*simple_fold)(uint32_t);
    } utf8char;
    struct {
        void* (*_indices_new_boxed)();
        bool (*_indices_push_boxed_entry)(void*, void*);
        size_t (*indices_len)(void*);
        void* (*indices_at)(void*, size_t);
        void* (*_entry_as_const)(void*);
        void* (*_entry_new_boxed)();
        void* (*entry_new_cloned)(const void*);
        bool (*entry_levels_push)(void*, const uint8_t*, size_t, const uint8_t*, size_t);
        bool (*entry_range_set)(void*, const uint8_t*, size_t);
        bool (*entry_page_commands_set)(void*, const uint8_t*, size_t);
        bool (*entry_pages_push)(void*, const uint8_t*, size_t, uint64_t);
        bool (*entry_pages_raw_set)(void*, const uint8_t*, size_t);
        size_t (*entry_levels_count)(void*);
        bool (*entry_key_n)(void*, size_t, StrRef*);
        bool (*entry_is_same)(const void*, const void*);
        size_t (*merged_levels_count)(void*);
        bool (*merged_key_n)(void*, size_t, StrRef*);
        bool (*merged_level_n)(void*, size_t, StrRef*);
        size_t (*merged_pages_count)(void*);
        bool (*merged_page_command_n)(void*, size_t, StrRef*);
        bool (*merged_pages_raw_n)(void*, size_t, StrRef*, StrRef*);
        size_t (*merged_page_span_n)(void*, size_t);
        size_t (*merged_max_same_key)(void*, const void*);
    } index;
    struct {
        bool (*has_han_data)(uint32_t);
        bool (*han_data)(uint32_t, CjkInfo*);
        int64_t (*original_radical)(const CjkInfo*);
        int64_t (*kx_radical_from_kind)(const uint8_t*, size_t);
        uint16_t (*kx_radical_to_index)(uint8_t);
        uint32_t (*kx_radical_char)(uint8_t);
        uint32_t (*kx_ideography_char)(uint8_t);
        bool (*kx_radical_kind)(uint8_t, StrRef*);
        bool (*mandarin)(const CjkInfo*, StrRef*, uint8_t*);
        bool (*cantonese)(const CjkInfo*, StrRef*, uint8_t*);
        bool (*han_ordered_strokes)(uint32_t, OrderedStrokes*);
        size_t (*han_ordered_strokes_len)(const OrderedStrokes*);
        uint8_t (*han_ordered_stroke_n)(const OrderedStrokes*, size_t);
        int64_t (*han_ordered_strokes_cmp_with_str)(const OrderedStrokes*, const uint8_t*, size_t);

        void* (*emoji)(const uint8_t*, size_t);
        bool (*emoji_str)(void*, StrRef*);
        bool (*emoji_name)(void*, StrRef*);
        uint8_t (*emoji_group)(void*);
    } data;
    struct {
        pcre2_code_8* (*pcre2_compile_8)(const uint8_t*, size_t, uint32_t, int*, size_t*, void*);
        int (*pcre2_jit_compile_8)(pcre2_code_8*, uint32_t);
        pcre2_match_data_8* (*pcre2_match_data_create_from_pattern_8)(const pcre2_code_8*, pcre2_general_context_8*);
        int (*pcre2_match_8)(const pcre2_code_8*, const uint8_t*, size_t, size_t, uint32_t, pcre2_match_data_8*, pcre2_match_context_8*);
        int (*pcre2_substitute_8)(const pcre2_code_8*, const uint8_t*, size_t, size_t, uint32_t, pcre2_match_data_8*, pcre2_match_context_8*, const uint8_t*, size_t, uint8_t*, size_t*);
        int (*pcre2_pattern_info_8)(const pcre2_code_8*, uint32_t, void*);
        size_t* (*pcre2_get_ovector_pointer_8)(pcre2_match_data_8*);
        void (*pcre2_code_free_8)(pcre2_code_8*);
        void (*pcre2_match_data_free_8)(pcre2_match_data_8*);
        int (*pcre2_get_error_message_8)(int, uint8_t*, size_t);
    } pcre2;
} Api;
]]

local __api = ffi.cast("Api*", __api);

local OutU8 = ffi.typeof("uint8_t[1]");
local OutInt = ffi.typeof("int[1]");
local OutSize = ffi.typeof("size_t[1]");
local OutU32 = ffi.typeof("uint32_t[1]");
local OutU8Ptr = ffi.typeof("unsigned char*[1]");
local uint32_t = ffi.typeof("uint32_t");
local size_t = ffi.typeof("size_t");
local voidptr_t = ffi.typeof("void*");
local char_buf_t = ffi.typeof("const uint8_t*");
local strptr_t = ffi.typeof("const StrRef*");
local U8VLA = ffi.typeof("uint8_t[?]");

local band, bor, lshift = bit.band, bit.bor, bit.lshift;
local byte = string.byte;


Logger = {
    error = function (s)
        if type(s) == "string" then return __api.log(1, s, #s);
        elseif ffi.istype(StrRef, s) then return __api.log(1, s.ptr, s.len);
        else error("invalid argument type for Logger.error, except StrRef|string, got " .. type(s));
        end
    end,
    warn = function (s)
        if type(s) == "string" then return __api.log(2, s, #s);
        elseif ffi.istype(StrRef, s) then return __api.log(2, s.ptr, s.len);
        else error("invalid argument type for Logger.warn, except StrRef|string, got " .. type(s));
        end
    end,
    info = function (s)
        if type(s) == "string" then return __api.log(3, s, #s);
        elseif ffi.istype(StrRef, s) then return __api.log(3, s.ptr, s.len);
        else error("invalid argument type for Logger.info, except StrRef|string, got " .. type(s));
        end
    end,
    debug = function (s)
        if type(s) == "string" then return __api.log(4, s, #s);
        elseif ffi.istype(StrRef, s) then return __api.log(4, s.ptr, s.len);
        else error("invalid argument type for Logger.debug, except StrRef|string, got " .. type(s));
        end
    end,
    trace = function (s)
        if type(s) == "string" then return __api.log(5, s, #s);
        elseif ffi.istype(StrRef, s) then return __api.log(5, s.ptr, s.len);
        else error("invalid argument type for Logger.trace, except StrRef|string, got " .. type(s));
        end
    end,
}
IstInputStyle = {
    is_nil = function() return __api.ist_input == ffi.cast(voidptr_t, 0) end,
    keyword = function() local res = ffi.new(StrRef);
        if __api.ist_input_fn.keyword(__api.ist_input, res) then return res else return nil end
    end,
    arg_open = function() return ffi.new(Char, __api.ist_input_fn.arg_open(__api.ist_input)) end,
    arg_close = function() return ffi.new(Char, __api.ist_input_fn.arg_close(__api.ist_input)) end,
    range_open = function() return ffi.new(Char, __api.ist_input_fn.range_open(__api.ist_input)) end,
    range_close = function() return ffi.new(Char, __api.ist_input_fn.range_close(__api.ist_input)) end,
    level = function() return ffi.new(Char, __api.ist_input_fn.level(__api.ist_input)) end,
    actual = function() return ffi.new(Char, __api.ist_input_fn.actual(__api.ist_input)) end,
    encap = function() return ffi.new(Char, __api.ist_input_fn.encap(__api.ist_input)) end,
    quote = function() return ffi.new(Char, __api.ist_input_fn.quote(__api.ist_input)) end,
    escape = function() return ffi.new(Char, __api.ist_input_fn.escape(__api.ist_input)) end,
    page_compositor = function() local res = ffi.new(StrRef);
        if __api.ist_input_fn.page_compositor(__api.ist_input, res) then return res else return nil end
    end,
    comment = function() return ffi.new(Char, __api.ist_input_fn.comment(__api.ist_input)) end,
    separator = function() local res = ffi.new(StrRef);
        if __api.ist_input_fn.separator(__api.ist_input, res) then return res else return nil end
    end,
};
IstOutputStyle = {
    is_nil = function() return __api.ist_output == ffi.cast(voidptr_t, 0) end,
    preamble = function() local res = ffi.new(StrRef);
        if __api.ist_output_fn.preamble(__api.ist_output, res) then return res else return nil end
    end,
    postamble = function() local res = ffi.new(StrRef);
        if __api.ist_output_fn.postamble(__api.ist_output, res) then return res else return nil end
    end,
    group_skip = function() local res = ffi.new(StrRef);
        if __api.ist_output_fn.group_skip(__api.ist_output, res) then return res else return nil end
    end,
    heading_prefix = function() local res = ffi.new(StrRef);
        if __api.ist_output_fn.heading_prefix(__api.ist_output, res) then return res else return nil end
    end,
    heading_suffix = function() local res = ffi.new(StrRef);
        if __api.ist_output_fn.heading_suffix(__api.ist_output, res) then return res else return nil end
    end,
    headings_flag = function() return tonumber(__api.ist_output_fn.headings_flag(__api.ist_output)) end,
    numhead_positive = function() local res = ffi.new(StrRef);
        if __api.ist_output_fn.numhead_positive(__api.ist_output, res) then return res else return nil end
    end,
    numhead_negative = function() local res = ffi.new(StrRef);
        if __api.ist_output_fn.numhead_negative(__api.ist_output, res) then return res else return nil end
    end,
    symhead_positive = function() local res = ffi.new(StrRef);
        if __api.ist_output_fn.symhead_positive(__api.ist_output, res) then return res else return nil end
    end,
    symhead_negative = function() local res = ffi.new(StrRef);
        if __api.ist_output_fn.symhead_negative(__api.ist_output, res) then return res else return nil end
    end,
    item_0 = function() local res = ffi.new(StrRef);
        if __api.ist_output_fn.item_0(__api.ist_output, res) then return res else return nil end
    end,
    item_1 = function() local res = ffi.new(StrRef);
        if __api.ist_output_fn.item_1(__api.ist_output, res) then return res else return nil end
    end,
    item_2 = function() local res = ffi.new(StrRef);
        if __api.ist_output_fn.item_2(__api.ist_output, res) then return res else return nil end
    end,
    item_01 = function() local res = ffi.new(StrRef);
        if __api.ist_output_fn.item_01(__api.ist_output, res) then return res else return nil end
    end,
    item_x1 = function() local res = ffi.new(StrRef);
        if __api.ist_output_fn.item_x1(__api.ist_output, res) then return res else return nil end
    end,
    item_12 = function() local res = ffi.new(StrRef);
        if __api.ist_output_fn.item_12(__api.ist_output, res) then return res else return nil end
    end,
    item_x2 = function() local res = ffi.new(StrRef);
        if __api.ist_output_fn.item_x2(__api.ist_output, res) then return res else return nil end
    end,
    delim_0 = function() local res = ffi.new(StrRef);
    if __api.ist_output_fn.delim_0(__api.ist_output, res) then return res else return nil end
end,
    delim_1 = function() local res = ffi.new(StrRef);
        if __api.ist_output_fn.delim_1(__api.ist_output, res) then return res else return nil end
    end,
    delim_2 = function() local res = ffi.new(StrRef);
        if __api.ist_output_fn.delim_2(__api.ist_output, res) then return res else return nil end
    end,
    delim_n = function() local res = ffi.new(StrRef);
        if __api.ist_output_fn.delim_n(__api.ist_output, res) then return res else return nil end
    end,
    delim_r = function() local res = ffi.new(StrRef);
        if __api.ist_output_fn.delim_r(__api.ist_output, res) then return res else return nil end
    end,
    delim_t = function() local res = ffi.new(StrRef);
        if __api.ist_output_fn.delim_t(__api.ist_output, res) then return res else return nil end
    end,
    encap_prefix = function() local res = ffi.new(StrRef);
        if __api.ist_output_fn.encap_prefix(__api.ist_output, res) then return res else return nil end
    end,
    encap_infix = function() local res = ffi.new(StrRef);
        if __api.ist_output_fn.encap_infix(__api.ist_output, res) then return res else return nil end
    end,
    encap_suffix = function() local res = ffi.new(StrRef);
        if __api.ist_output_fn.encap_suffix(__api.ist_output, res) then return res else return nil end
    end,
    page_precedence = function() local res = ffi.new(StrRef);
        if __api.ist_output_fn.page_precedence(__api.ist_output, res) then return res else return nil end
    end,
    suffix_2p = function() local res = ffi.new(StrRef);
        if __api.ist_output_fn.suffix_2p(__api.ist_output, res) then return res else return nil end
    end,
    suffix_3p = function() local res = ffi.new(StrRef);
        if __api.ist_output_fn.suffix_3p(__api.ist_output, res) then return res else return nil end
    end,
    suffix_mp = function() local res = ffi.new(StrRef);
        if __api.ist_output_fn.suffix_mp(__api.ist_output, res) then return res else return nil end
    end,
    stroke_prefix = function() local res = ffi.new(StrRef);
        if __api.ist_output_fn.stroke_prefix(__api.ist_output, res) then return res else return nil end
    end,
    stroke_suffix = function() local res = ffi.new(StrRef);
        if __api.ist_output_fn.stroke_suffix(__api.ist_output, res) then return res else return nil end
    end,
    radical_prefix = function() local res = ffi.new(StrRef);
        if __api.ist_output_fn.radical_prefix(__api.ist_output, res) then return res else return nil end
    end,
    radical_suffix = function() local res = ffi.new(StrRef);
        if __api.ist_output_fn.radical_suffix(__api.ist_output, res) then return res else return nil end
    end,
    radical_simplified_flag = function() return tonumber(__api.ist_output_fn.radical_simplified_flag(__api.ist_output)) end,
    radical_simplified_prefix = function() local res = ffi.new(StrRef);
        if __api.ist_output_fn.radical_simplified_prefix(__api.ist_output, res) then return res else return nil end
    end,
    radical_simplified_delimiter = function() local res = ffi.new(StrRef);
        if __api.ist_output_fn.radical_simplified_delimiter(__api.ist_output, res) then return res else return nil end
    end,
    radical_simplified_suffix = function() local res = ffi.new(StrRef);
        if __api.ist_output_fn.radical_simplified_suffix(__api.ist_output, res) then return res else return nil end
    end,
};

local ordered_strokes_mt = {
    get = function (self, n)
        local s = __api.data.han_ordered_stroke_n(self, n);
        if s > 0 and s < 255 then
            return tonumber(s);
        else
            return nil;
        end
    end,
    cmp = function (lhs, rhs)
        local res = __api.strref.cmp(lhs, rhs);
        if res >= -1 and res <= 1 then
            return tonumber(res);
        else
            error("invalid OrderedStrokes");
        end
    end,
    cmp_with_str = function (lhs, rhs)
        local res = nil;
        if ffi.istype(OrderedStrokes, lhs) and type(rhs) == "string" then
            res = __api.data.han_ordered_strokes_cmp_with_str(lhs, rhs, #rhs);
        elseif type(lhs) == "string" and ffi.istype(OrderedStrokes, rhs) then
            res = __api.data.han_ordered_strokes_cmp_with_str(rhs, lhs, #lhs);
            if res >= -1 then
                res = -res;
            end
        end
        if not res then
            error("incompatible argument type for OrderedStrokes.cmp_with_str, except (OrderedStrokes, string)");
        elseif res < -1 then
            error("invalid OrderedStrokes");
        else
            return tonumber(res);
        end
    end,
};
local orderedstrokes_to_string = __c_function_api.orderedstrokes_to_string;
OrderedStrokes = ffi.metatype("OrderedStrokes", {
    __len = function (self) return tonumber(__api.data.han_ordered_strokes_len(self)) end,
    __tostring = function (self)
        local s = orderedstrokes_to_string(tonumber(ffi.cast(size_t, self.ptr)), tonumber(self.len));
        if s then return s else error("invalid OrderedStrokes") end
    end,
    __eq = function (lhs, rhs)
        if type(rhs) == "nil" then
            return lhs.ptr == ffi.cast(voidptr_t, 0);
        elseif type(lhs) == "nil" then
            return rhs.ptr == ffi.cast(voidptr_t, 0);
        else
            return __api.strref.cmp(lhs, rhs) == 0
        end
    end,
    __lt = function (lhs, rhs) return __api.strref.cmp(lhs, rhs) < 0 end,
    __le = function (lhs, rhs) return __api.strref.cmp(lhs, rhs) <= 0 end,
    __index = function (self, key)
        if type(key) ~= "string" then
            local s = __api.data.han_ordered_stroke_n(self, key);
            if s > 0 and s < 255 then
                return tonumber(s);
            else
                return nil;
            end
        else
            return ordered_strokes_mt[key];
        end
    end,
    __ipairs = function (self)
        local call_fn = function (os_, i)
            local res = __api.data.han_ordered_stroke_n(os_, i);
            if res > 0 and res < 255 then
                return i + 1, tonumber(res);
            else
                return nil;
            end
        end;
        return call_fn, self, 0;
    end,
    __new = nil,
});

local char_utf8_buf_t = ffi.typeof("uint8_t[5]");
Char = ffi.metatype("Char", {
    __eq = function (lhs, rhs) return Char.cmp(lhs, rhs) == 0 end,
    __lt = function (lhs, rhs) return Char.cmp(lhs, rhs) == -1 end,
    __le = function (lhs, rhs) return Char.cmp(lhs, rhs) ~= 1 end,
    __tostring = function (self)
        local buf = char_utf8_buf_t();
        if __api.utf8char.encode_utf8(self.inner, buf) then
            return ffi.string(buf);
        else
            error("invalid UTF-8 slot: U+" .. string.format("%X", self.inner));
        end
    end,
    __index = {
        is_valid = function (self)
            return __api.utf8char.is_valid(self.inner);
        end,
        from_number = function (n)
            return ffi.new(Char, __api.utf8char.from_i64(n));
        end,
        to_number = function (self) return tonumber(self.inner) end,
        cmp = function (lhs, rhs)
            if ffi.istype(Char, lhs) and ffi.istype(Char, rhs) then
                if not lhs:is_valid() or not rhs:is_valid() then
                    local s;
                    if lhs:is_valid() then
                        s = string.format("%X", rhs.inner);
                    else
                        s = string.format("%X", lhs.inner);
                    end
                    error("invalid UTF-8 slot: U+" .. s);
                end

                if lhs.inner < rhs.inner then
                    return -1;
                elseif lhs.inner == rhs.inner then
                    return 0;
                else
                    return 1;
                end
            elseif (ffi.istype(Char, lhs) and type(rhs) == "nil") then
                return __api.utf8char.is_valid(lhs.inner) == false and 0 or nil;
            elseif type(lhs) == "nil" and ffi.istype(Char, rhs) then
                return not __api.utf8char.is_valid(rhs.inner) == false and 0 or nil;
            else
                error("incompatible type to compare");
            end
        end,
        len_utf8 = function (self) return __api.utf8char.len_utf8(self.inner) end,

        from_string = function (s)
            local c = nil;
            for c_ in Char.chars(s) do
                if ffi.istype(Char, c) then
                    return c, false;
                else
                    c = c_;
                end
            end
            return c, ffi.istype(Char, c);
        end,
        chars = function(s)
            local i = 1;
            if type(s) ~= "string" then
                error("invalid argument #1 of Char.chars, except string");
            end

            local n = #s;
            return function()
                if i > n then return nil end
                local b1 = byte(s, i);
                if b1 < 0x80 then
                    i = i + 1;
                    return Char(b1);
                end
                if b1 >= 0xC2 and b1 <= 0xDF then
                    if i + 1 > n then return nil end
                    local b2 = byte(s, i + 1);
                    if band(b2, 0xC0) ~= 0x80 then return nil end
                    local cp = bor(lshift(band(b1, 0x1F), 6), band(b2, 0x3F));
                    i = i + 2;
                    return Char(cp)
                end
                if b1 >= 0xE0 and b1 <= 0xEF then
                    if i + 2 > n then return nil end
                    local b2 = byte(s, i + 1);
                    local b3 = byte(s, i + 2);
                    if band(b2, 0xC0) ~= 0x80 or band(b3, 0xC0) ~= 0x80 then return nil end
                    if b1 == 0xE0 and b2 < 0xA0 then return nil end
                    if b1 == 0xED and b2 >= 0xA0 then return nil end
                    local cp = bor(
                        lshift(band(b1, 0x0F), 12),
                        lshift(band(b2, 0x3F), 6),
                        band(b3, 0x3F)
                    );
                    i = i + 3;
                    return Char(cp);
                end
                if b1 >= 0xF0 and b1 <= 0xF4 then
                    if i + 3 > n then return nil end
                    local b2 = byte(s, i + 1);
                    local b3 = byte(s, i + 2);
                    local b4 = byte(s, i + 3);
                    if band(b2, 0xC0) ~= 0x80 or
                    band(b3, 0xC0) ~= 0x80 or
                    band(b4, 0xC0) ~= 0x80 then
                        return nil
                    end
                    if b1 == 0xF0 and b2 < 0x90 then return nil end
                    if b1 == 0xF4 and b2 > 0x8F then return nil end
                    local cp = bor(
                        lshift(band(b1, 0x07), 18),
                        lshift(band(b2, 0x3F), 12),
                        lshift(band(b3, 0x3F), 6),
                        band(b4, 0x3F)
                    );
                    i = i + 4;
                    return Char(cp);
                end
                return nil;
            end
        end,
        has_han_info = function(self) return __api.data.has_han_data(self.inner) end,
        han_info = function(self)
            local info = ffi.new(CjkInfo);
            if __api.data.han_data(self.inner, info) then
                return info;
            else
                return nil;
            end
        end,
        han_ordered_strokes = function (self) local res = ffi.new(OrderedStrokes);
            if __api.data.han_ordered_strokes(self.inner, res) then return res else return nil end
        end,
        script = function (self) return tonumber(__api.utf8char.script(self.inner)) end,
        general_category = function (self) return tonumber(__api.utf8char.general_category(self.inner)) end,
        is_alphabetic = function (self) return __api.utf8char.is_alphabetic(self.inner) end,
        is_numeric = function (self) return __api.utf8char.is_numeric(self.inner) end,
        is_math = function (self) return __api.utf8char.is_math(self.inner) end,
        is_ideographic = function (self) return __api.utf8char.is_ideographic(self.inner) end,
        simple_fold = function (self) return Char(__api.utf8char.simple_fold(self.inner)) end,
    },
    __new = function (_ct, n)
        if ffi.istype(Char, n) then
            return ffi.new(Char, n);
        elseif type(n) == "string" then
            local res = Char.from_string(n);
            return res;
        else
            return ffi.new(Char, __api.utf8char.from_i64(n));
        end
    end
});

StrRef = ffi.metatype("StrRef", {
    __tostring = function(self)
        if self:is_nil() then
            error("StrRef cannot be nil");
        else
            return ffi.string(self.ptr, self.len)
        end
    end,
    __eq = function (lhs, rhs)
        if type(rhs) == "nil" then
            return lhs.ptr == ffi.cast(voidptr_t, 0);
        elseif type(lhs) == "nil" then
            return rhs.ptr == ffi.cast(voidptr_t, 0);
        else
            return StrRef.cmp(lhs, rhs) == 0
        end
    end,
    __lt = function (lhs, rhs) return StrRef.cmp(lhs, rhs) == -1 end,
    __le = function (lhs, rhs) return StrRef.cmp(lhs, rhs) ~= 1 end,
    __len = function(self) return tonumber(self.len) end,
    __index = {
        is_nil = function (self) return ffi.cast(voidptr_t, self.ptr) == ffi.cast(voidptr_t, 0) end,
        is_empty = function (self) return self:is_nil() or self.len == 0 end,
        from_ptr = function (ptr)
            if ffi.cast(voidptr_t, ptr) == ffi.cast(voidptr_t, 0) then
                return nil;
            else
                return StrRef(ffi.cast(strptr_t, ptr)[0]);
            end
        end,
        unsafe_new = function (s, len)
            local res = ffi.new(StrRef);
            local ok = false;
            if ffi.istype(StrRef, s) then
                ok = __api.strref.from_buf(s.ptr, s.len, res);
            elseif type(s) == "string" then
                ok = __api.strref.from_buf(s, #s, res);
            elseif ffi.istype(char_buf_t, s) then
                ok = __api.strref.from_buf(s, len or __api.strref.strlen(s), res);
            else
                error("invalid argument for StrRef.unsafe_new, except StrRef|string|const uint8_t*");
            end
            if ok then return res else return nil end
        end,
        to_string_escaped = function (s, all)
            s = StrRef.unsafe_new(s);
            local s_buf = string_buffer.new();
            if all then
                for c in s:chars() do
                    s_buf:putf("\\u{%04X}", c);
                end
            else
                local buf = ffi.new(char_utf8_buf_t);
                local chr;
                for c in s:chars() do
                    buf[0], buf[1], buf[2], buf[3], buf[4] = 0, 0, 0, 0, 0;
                    chr = Char(c);
                    if __api.utf8char._is_visible(chr.inner) then
                        if __api.utf8char.encode_utf8(chr.inner, buf) then
                            s_buf:putcdata(buf, chr:len_utf8());
                        end
                    else
                        s_buf:putf("\\u{%04X}", c);
                    end
                end
            end
            return s_buf:tostring();
        end,
        cmp = function (lhs, rhs)
            local res = nil;
            if ffi.istype(StrRef, lhs) and ffi.istype(StrRef, rhs) then
                res = __api.strref.cmp(lhs, rhs);
            elseif ffi.istype(StrRef, lhs) and type(rhs) == "string" then
                res = __api.strref.cmp_buf(lhs, rhs, #rhs);
            elseif type(lhs) == "string" and ffi.istype(StrRef, rhs) then
                res = __api.strref.cmp_buf(rhs, lhs, #lhs);
                if res >= -1 then
                    res = -res;
                end
            end
            if res == nil then
                error("incompatible type to compare");
            elseif res >= -1 then
                return tonumber(res);
            else
                error("invalid StrRef");
            end
        end,
        eq_ignore_case = function (lhs, rhs) return StrRef.cmp_ignore_case(lhs, rhs) == 0 end;
        cmp_ignore_case = function (lhs, rhs)
            local res = nil;
            if ffi.istype(StrRef, lhs) and ffi.istype(StrRef, rhs) then
                res = __api.strref.cmp_ignore_case(lhs.ptr, lhs.len, rhs.ptr, rhs.len);
            elseif ffi.istype(StrRef, lhs) and type(rhs) == "string" then
                res = __api.strref.cmp_ignore_case(lhs.ptr, lhs.len, rhs, #rhs);
            elseif type(lhs) == "string" and ffi.istype(StrRef, rhs) then
                res = __api.strref.cmp_ignore_case(lhs, #lhs, rhs.ptr, rhs.len);
            elseif type(lhs) == "string" and type(rhs) == "string" then
                res = __api.strref.cmp_ignore_case(lhs, #lhs, rhs, #rhs);
            elseif type(lhs) == "nil" and (ffi.istype(StrRef, rhs) or type(rhs) == "string") then
                return nil;
            elseif (ffi.istype(StrRef, lhs) or type(lhs) == "string") and type(rhs) == "nil" then
                return nil;
            end
            if res == nil then
                error("incompatible type of StrRef.cmp_ignore_case");
            elseif res >= -1 and res <= 1 then
                return tonumber(res);
            else
                error("invalid StrRef");
            end
        end,
        slice = function (self, start_idx, end_idx)
            local res = ffi.new(StrRef);
            local ok;
            if end_idx == nil then
                ok = __api.strref.slice(self, 0, start_idx, res);
            else
                ok = __api.strref.slice(self, start_idx, end_idx, res);
            end
            if ok then
                return res;
            else
                error("invalid slice for StrRef[" .. start_idx .. ".." .. end_idx .. "]")
            end
        end,
        head = function (s)
            -- local new_s = ffi.new(StrRef, self);
            -- local res = tonumber(__api.strref.char_next(new_s));
            local res;
            if ffi.istype(StrRef, s) then
                res = __api.strref.head(s.ptr);
            else
                res = __api.strref.head(s);
            end
            if res > 0x10FFFF then return nil else return tonumber(res) end
        end,
        chars = function (self)
            local new_s = ffi.new(StrRef, self);
            return function ()
                local res = tonumber(__api.strref.char_next(new_s));
                if res > 0x10FFFF then
                    return nil;
                else
                    return res;
                end
            end;
        end,
        lines = function (self)
            local new_s = ffi.new(StrRef, self);
            return function ()
                local res = ffi.new(StrRef);
                if __api.strref.line_next(new_s, res) then
                    return res;
                else
                    return nil;
                end
            end;
        end,
        split = function (self, pat)
            local new_s = ffi.new(StrRef, self);
            local call_fn;
            if ffi.istype(Char, pat) then
                call_fn = function () local res = ffi.new(StrRef);
                    if __api.strref.split_char_next(new_s, pat.inner, res) then return res else return nil end;
                end;
            elseif ffi.istype(StrRef, pat) then
                call_fn = function() local res = ffi.new(StrRef);
                    if __api.strref.split_str_next(new_s, pat.ptr, pat.len, res) then return res else return nil end end;
            elseif type(pat) == "string" then
                call_fn = function() local res = ffi.new(StrRef);
                    if __api.strref.split_str_next(new_s, pat, #pat, res) then return res else return nil end end;
            elseif type(pat) == "nil" then
                call_fn = function () local res = ffi.new(StrRef);
                    if __api.strref.split_spaces_next(new_s, res) then return res else return nil end end;
            else
                error("in valid pattern to split StrRef");
            end
            return call_fn;
        end,
        graphemes = function (self)
            local new_s = ffi.new(StrRef, self);
            return function ()
                local res = ffi.new(StrRef);
                if __api.strref.graphemes_next(new_s, res) then
                    return res;
                else
                    return nil;
                end
            end;
        end,
        unicode_words = function (self)
            local new_s = ffi.new(StrRef, self);
            return function ()
                local res = ffi.new(StrRef);
                if __api.strref.unicode_words_next(new_s, res) then
                    return res;
                else
                    return nil;
                end
            end;
        end,
        split_word_bounds = function (self)
            local new_s = ffi.new(StrRef, self);
            return function ()
                local res = ffi.new(StrRef);
                if __api.strref.split_word_bounds_next(new_s, res) then
                    return res;
                else
                    return nil;
                end
            end;
        end,
        trim_ascii_spaces = function (self, start, end_)
            local out = ffi.new(StrRef, self);
            if start ~= false then start = true end
            if end_ ~= false then end_ = true end
            if __api.strref.trim_ascii_spaces(self, start, end_, out) then
                return out;
            else
                return nil;
            end
        end,
        is_emoji = function (s) return not (__api.data.emoji(s.ptr, s.len) == ffi.cast(voidptr_t, 0)) end,
    },
    __newindex = function (_table, key, value)
        error("unable to set value to StrRef[" .. key .. "]=" .. value);
    end,
});

local index_entry_array_mt = {
    is_nil = function (arr)
        return arr.inner == ffi.cast(voidptr_t, 0);
    end,
    is_empty = function (self)
        return self.inner == ffi.cast(voidptr_t, 0) or __api.index.indices_len(self.inner) == 0;
    end,
    get = function (arr, n)
        return ffi.new(IndexEntryRef, __api.index.indices_at(arr, n));
    end,
    push = function (self, entry)
        if ffi.istype(IndexEntryRefMut, entry) then
            __api.index._indices_push_boxed_entry(self.inner, entry.inner);
        elseif type(entry) == "table" then
            __api.index._indices_push_boxed_entry(self.inner, IndexEntryRefMut(entry).inner);
        else
            error("invalid argument type for IndexEntryArray.push, expect IndexEntryRefMut|table");
        end
    end,
    extend_from_iterator = function (self, f)
        if not ffi.istype(IndexEntryArray, self) then
            error("invalid argument for IndexEntryArray.extend_from_iterator");
        end

        local arr_ptr = self.inner;
        for entry in f() do
            if ffi.istype(IndexEntryRefMut, entry) then
                __api.index._indices_push_boxed_entry(arr_ptr, entry.inner);
            elseif type(entry) == "table" then
                __api.index._indices_push_boxed_entry(arr_ptr, IndexEntryRefMut(entry).inner);
            elseif entry == false then
                -- pass
            else
                error("invalid return value of parse_index_line, except IndexEntryRefMut|table, got " .. type(entry));
            end
        end
    end
}
IndexEntryArray = ffi.metatype("IndexEntryArray", {
    __len = function (arr)
        return __api.index.indices_len(arr.inner);
    end,
    __index = function (arr, key)
        if type(key) ~= "string" then
            return ffi.new(IndexEntryRef, __api.index.indices_at(arr.inner, key));
        else
            return index_entry_array_mt[key];
        end
    end,
    __ipairs = function (arr)
        local call_fn = function (a, i)
            local res = ffi.new(IndexEntryRef, __api.index.indices_at(a, i));
            if res:is_nil() then
                return nil;
            else
                return i + 1, res;
            end
        end;
        return call_fn, arr, 0;
    end,
    __new = nil,
});

local indexentry_to_table = __c_function_api.indexentry_to_table;
IndexEntryRef = ffi.metatype("IndexEntryRef", {
    __eq = function (lhs, rhs)
        if type(lhs) == "nil" and ffi.istype(IndexEntryRef, rhs) then
            return nil;
        elseif ffi.istype(IndexEntryRef, lhs) and type(rhs) == "nil" then
            return nil;
        elseif ffi.istype(IndexEntryRef, lhs) and ffi.istype(IndexEntryRef, rhs) then
            return lhs.inner == rhs.inner;
        else
            error("incompatible type for IndexEntryRef.__eq");
        end
    end,
    __index = {
        is_nil = function (e)
            return e.inner == ffi.cast(voidptr_t, 0);
        end,
        to_handle = function (self)
            if not ffi.istype(IndexEntryRef, self) or self:is_nil() then
                return nil;
            else
                return tonumber(self.inner);
            end
        end,
        levels_count = function (self)
            return tonumber(__api.index.entry_levels_count(self.inner));
        end,
        key_n = function (entry, n) local res = ffi.new(StrRef);
            if __api.index.entry_key_n(entry.inner, n, res) then return res else return nil end
        end,
        keys = function (self)
            local i = 0;
            local call_fn = function ()
                local res = self:key_n(i);
                if res == nil then
                    return nil;
                else
                    i = i + 1;
                    return res;
                end
            end;
            return call_fn;
        end,
        as_table = function (self)
            local ptr = ffi.cast(size_t, self.inner);
            return indexentry_to_table(tonumber(ptr));
        end,
        is_same = function (lhs, rhs)
            if ffi.istype(IndexEntryRef, lhs) and ffi.istype(IndexEntryRef, rhs) then
                return __api.index.is_same(lhs.inner, rhs.inner);
            elseif type(lhs) == "nil" or type(rhs) == "nil" then
                return nil;
            else
                error("invalid argument for IndexEntryRef.is_same, except (IndexEntryRef, IndexEntryRef)");
            end
        end,
    },
    __new = nil,
});
IndexEntryRefMut = ffi.metatype("IndexEntryRefMut", {
    __index = {
        push_level = function (self, key, print)
            if type(key) == "nil" and type(print) == "string" then
                return __api.index.entry_levels_push(self.inner, ffi.cast(voidptr_t, 0), 0, print, #print);
            elseif type(key) == "string" and type(print) == "nil" then
                return __api.index.entry_levels_push(self.inner, ffi.cast(voidptr_t, 0), 0, key, #key);
            elseif type(key) == "string" and type(print) == "string" then
                return __api.index.entry_levels_push(self.inner, key, #key, print, #print);
            elseif type(key) == "nil" and ffi.istype(StrRef, print) then
                return __api.index.entry_levels_push(self.inner, ffi.cast(voidptr_t, 0), 0, print.ptr, print.len);
            elseif ffi.istype(StrRef, key) and type(print) == "nil" then
                return __api.index.entry_levels_push(self.inner, ffi.cast(voidptr_t, 0), 0, key.ptr, key.len);
            elseif ffi.istype(StrRef, key) and ffi.istype(StrRef, print) then
                return __api.index.entry_levels_push(self.inner, key.ptr, key.len, print.ptr, print.len);
            else
                error("incompatible types for IndexEntryRefMut.push_level, except: (StrRef|string|nil, StrRef|string|nil)");
            end
        end,
        set_range = function (self, range)
            if type(range) == "string" then
                return __api.index.entry_range_set(self.inner, range, #range);
            elseif ffi.istype(StrRef, range) then
                return __api.index.entry_range_set(self.inner, range.ptr, range.len);
            elseif type(range) == "nil" then
                return __api.index.entry_range_set(self.inner, "nil", 3);
            else
                error("incompatible types for IndexEntryRefMut.set_range, except: StrRef|string|nil");
            end
        end,
        set_page_commands = function (self, page_commands)
            if type(page_commands) == "string" then
                return __api.index.entry_page_commands_set(self.inner, page_commands, #page_commands);
            elseif ffi.istype(StrRef, page_commands) then
                return __api.index.entry_page_commands_set(self.inner, page_commands.ptr, page_commands.len);
            elseif type(page_commands) == "nil" then
                return __api.index.entry_page_commands_set(self.inner, ffi.cast(voidptr_t, 0), 0);
            else
                error("incompatible types for IndexEntryRefMut.set_page_commands, except: StrRef|string|nil");
            end
        end,
        push_page = function (self, kind, n)
            local num = tonumber(n);
            if type(kind) == "string" then
                return __api.index.entry_pages_push(self.inner, kind, #kind, num);
            elseif ffi.istype(StrRef, kind) then
                return __api.index.entry_pages_push(self.inner, kind.ptr, kind.len, num);
            else
                error("incompatible types for IndexEntryRefMut.push_page, except: (StrRef|string, number)");
            end
        end,
        set_pages_raw = function (self, raw)
            if type(raw) == "string" then
                return __api.index.entry_pages_raw_set(self.inner, raw, #raw);
            elseif ffi.istype(StrRef, raw) then
                return __api.index.entry_pages_raw_set(self.inner, raw.inner, raw.len);
            else
                error("incompatible types for IndexEntryRefMut.set_pages_raw, except: StrRef|string");
            end
        end,
        as_table = function (self)
            local e_const = __api.index._entry_as_const(self.inner);
            return ffi.new(IndexEntryRef, e_const):as_table();
        end,
    },
    __new = function (_ct, val)
        if ffi.istype(IndexEntryRef, val) or ffi.istype(IndexEntryRefMut, val) then
            return ffi.new(IndexEntryRefMut, __api.index.entry_new_cloned(val.inner));
        elseif type(val) == "table" then
            if not next(val) then return nil end
            if not val.levels then
                error("except field 'levels' of the table when creates IndexEntryRefMut");
            end

            local res = ffi.new(IndexEntryRefMut, __api.index._entry_new_boxed());
            for _, v in ipairs(val.levels) do
                if #v == 1 then
                    res:push_level(nil, v[1]);
                elseif #v >= 2 then
                    res:push_level(v[1], v[2]);
                end
            end
            res:set_range(val.range);
            res:set_page_commands(val.page_commands);
            if val.pages then
                for _, v in ipairs(val.pages) do
                    res:push_page(v.kind, v.value);
                end
            end
            if val.pages_raw then
                res:set_pages_raw(val.pages_raw);
            end
            return res;
        elseif type(val) ~= "nil" then
            error("constructor of IndexEntryRefMut requires a table or 0 argument");
        else
            return ffi.new(IndexEntryRefMut, __api.index._entry_new_boxed());
        end
    end,
});
local mergedentry_to_table = __c_function_api.mergedentry_to_table;
MergedEntryRef = ffi.metatype("MergedEntryRef", {
    __eq = function (lhs, rhs)
        if type(lhs) == "nil" and ffi.istype(IndexEntryRef, rhs) then
            return nil;
        elseif ffi.istype(IndexEntryRef, lhs) and type(rhs) == "nil" then
            return nil;
        elseif ffi.istype(IndexEntryRef, lhs) and ffi.istype(IndexEntryRef, rhs) then
            return lhs.inner == rhs.inner;
        else
            error("incompatible type for MergedEntryRef.__eq");
        end
    end,
    __index = {
        is_nil = function (e)
            return e.inner == ffi.cast(voidptr_t, 0);
        end,
        to_handle = function (self)
            if not ffi.istype(MergedEntryRef, self) or self:is_nil() then
                return nil;
            else
                return tonumber(self.inner);
            end
        end,
        levels_count = function (self)
            return tonumber(__api.index.merged_levels_count(self.inner));
        end,
        level_n = function (entry, n) local res = ffi.new(StrRef);
            if __api.index.merged_level_n(entry.inner, n, res) then return res else return nil end
        end,
        levels = function (self)
            local i = 0;
            local call_fn = function ()
                local res = self:level_n(i);
                if res == nil then
                    return nil;
                else
                    i = i + 1;
                    return res;
                end
            end;
            return call_fn;
        end,
        key_n = function (self, n) local res = ffi.new(StrRef);
            if __api.index.merged_key_n(self.inner, n, res) then return res else return nil end
        end,
        keys = function (self)
            local i = 0;
            local call_fn = function ()
                local res = self:key_n(i);
                if res == nil then
                    return nil;
                else
                    i = i + 1;
                    return res;
                end
            end;
            return call_fn;
        end,
        pages_count = function (self)
            return tonumber(__api.index.merged_pages_count(self.inner));
        end,
        page_command_n = function (self, n) local res = ffi.new(StrRef);
            if __api.index.merged_page_command_n(self.inner, n, res) then
                if res == nil then
                    return false;
                else
                    return res;
                end
            else
                return nil;
            end
        end,
        page_commands = function (self)
            local i = 0;
            local call_fn = function ()
                local res = self:page_command_n(i);
                if res == nil then
                    return nil;
                else
                    i = i + 1;
                    return res;
                end
            end;
            return call_fn;
        end,
        pages_raw_n = function (self, n)
            local o1, o2 = ffi.new(StrRef), ffi.new(StrRef);
            if __api.index.merged_pages_raw_n(self.inner, n, o1, o2) then
                if o2 == nil then
                    return o1, nil;
                else
                    return o1, o2;
                end
            else
                return nil, nil;
            end
        end,
        pages_raws = function (self)
            local i = 0;
            local call_fn = function ()
                local o1, o2 = self:pages_raw_n(i);
                if o1 == nil then
                    return nil, nil;
                else
                    i = i + 1;
                    return o1, o2;
                end
            end;
            return call_fn;
        end,
        page_span_n = function (self, n)
            return tonumber(__api.index.merged_page_span_n(self.inner, n));
        end,
        page_spans = function (self)
            local i = 0;
            local len = self:pages_count();
            local call_fn = function ()
                local res = self:page_span_n(i);
                i = i + 1;
                if i >= len then return nil else return res end
            end;
            return call_fn;
        end,
        max_same_key = function (entry1, entry2)
            return tonumber(__api.index.merged_max_same_key(entry1.inner, entry2.inner));
        end,
        as_table = function (self)
            local ptr = ffi.cast(size_t, self.inner);
            return mergedentry_to_table(tonumber(ptr));
        end
    },
    __new = nil,
});

CjkInfo = ffi.metatype("CjkInfo", {
    __tostring = function (info)
        local extra = string_buffer.new();
        local mandarin, m_tone = info:mandarin();
        local cantonese, k_tone = info:cantonese();
        if mandarin then
            extra:putf("mandarin: %s", mandarin);
            if m_tone ~= 0 then extra:put(tostring(m_tone)) end
        end
        if mandarin then
            if cantonese then extra:put(", ") end
            extra:putf("cantonese: %s", cantonese);
            if m_tone ~= 0 then extra:put(tostring(k_tone)) end
        end
        if mandarin or cantonese then extra:put(", ") end
        return string.format(
            "CjkInfo { radical: %s, additional_strokes: %d, k_total_strokes: %u, glyph_total_strokes: %u, %s.. }",
            info:kx_radical_kind(),
            info.additional_strokes,
            info.k_total_strokes,
            info.glyph_total_strokes,
            extra
        );
    end,
    __eq = function(lhs, rhs)
        if ffi.istype(CjkInfo, lhs) and type(rhs) == "nil" then
            return lhs.k_total_strokes == 0 or nil;
        elseif type(lhs) == "nil" and ffi.istype(CjkInfo, rhs) then
            return rhs.k_total_strokes == 0 or nil;
        elseif ffi.istype(CjkInfo, lhs) and ffi.istype(CjkInfo, rhs) then
            error("CjkInfo cannot be compared");
        else
            error("incompatible type to compare");
        end
    end,
    __index = {
        radical_from_kind = function (kind)
            local res;
            if ffi.istype(StrRef, kind) then
                res = __api.data.radical_from_kind(kind.ptr, kind.len);
            elseif type(kind) == "string" then
                res = __api.data.radical_from_kind(kind, #kind);
            else
                error("invalid argument type for CjkInfo.radical_from_kind, except StrRef|string");
            end
            if res < 255 then return tonumber(res) else return nil end
        end,
        is_nil = function (self) return self.k_total_strokes == 0 end,
        contains_char = function(chr) return Char(chr):has_han_info() end,
        kx_radical_char = function (self) return ffi.new(Char, __api.data.kx_radical_char(self.radical)) end,
        kx_ideography_char = function (self) return ffi.new(Char, __api.data.kx_ideography_char(self.radical)) end,
        kangxi_radical_number = function (self)
            local res = __api.data.kx_radical_to_index(self.radical);
            if res < 0xFFFF then
                return band(0x00FF, tonumber(res));
            else
                return nil;
            end
        end,
        kangxi_radical_ideography_slot = function (self)
            if self:is_nil() then
                return nil;
            else
                return tonumber(__api.data.kx_ideography_char(__api.data.original_radical(self)));
            end
        end,
        kx_radical_kind = function (self) local res = ffi.new(StrRef);
            if __api.data.kx_radical_kind(self.radical, res) then return res else return nil end
        end,
        ucd_additional_strokes = function (self) return tonumber(self.additional_strokes) end,
        ucd_total_strokes = function (self) return tonumber(self.k_total_strokes) end,
        total_strokes = function (self) return tonumber(self.glyph_total_strokes) end,
        original_radical = function (self)
            local idx = __api.data.original_radical(self);
            if idx < 0 then return nil else return tonumber(idx) end
        end,
        mandarin = function (self)
            local mandarin, tone = ffi.new(StrRef), ffi.new(OutU8);
            if __api.data.mandarin(self, mandarin, tone) then
                return mandarin, tonumber(tone[0]);
            else
                return nil, nil;
            end
        end,
        cantonese = function (self)
            local mandarin, tone = ffi.new(StrRef), ffi.new(OutU8);
            if __api.data.cantonese(self, mandarin, tone) then
                return mandarin, tonumber(tone[0]);
            else
                return nil, nil;
            end
        end,
    },
    __new = function (_ct, c) return Char(c):han_info() end,
});

Emoji = ffi.metatype("Emoji", {
    __tostring = function (self) local res = ffi.new(StrRef);
        if __api.data.emoji_str(self.inner, res) then return tostring(res) else return nil end end,
    __index = {
        name = function (self) local res = ffi.new(StrRef);
            if __api.data.emoji_name(self.inner, res) then return res else return nil end end,
        group = function (self) return tonumber(__api.data.emoji_group(self.inner)) end,
    },
    __new = function (_ct, s)
        local ptr = nil;
        if ffi.istype(StrRef, s) then
            ptr = __api.data.emoji(s.ptr, s.len);
        elseif type(s) == "string" then
            ptr = __api.data.emoji(s, #s);
        end
        if ptr and ptr ~= ffi.cast(voidptr_t, 0) then
            return ffi.new(Emoji, ptr);
        else
            return nil;
        end
    end,
});

local Pcre2_Constant = {
    UNSET = -1,
    ZERO_TERMINATED = -1,

    ALLOW_EMPTY_CLASS = 1,
    ALT_BSUX = 2,
    AUTO_CALLOUT = 4,
    CASELESS = 8,
    DOLLAR_ENDONLY = 16,
    DOTALL = 32,
    DUPNAMES = 64,
    EXTENDED = 128,
    FIRSTLINE = 256,
    MATCH_UNSET_BACKREF = 512,
    MULTILINE = 1024,
    NEVER_UCP = 2048,
    NEVER_UTF = 4096,
    NO_AUTO_CAPTURE = 8192,
    NO_AUTO_POSSESS = 16384,
    NO_DOTSTAR_ANCHOR = 32768,
    NO_START_OPTIMIZE = 65536,
    UCP = 131072,
    UNGREEDY = 262144,
    UTF = 524288,
    NEVER_BACKSLASH_C = 1048576,
    ALT_CIRCUMFLEX = 2097152,
    ALT_VERBNAMES = 4194304,
    USE_OFFSET_LIMIT = 8388608,
    EXTENDED_MORE = 16777216,
    LITERAL = 33554432,
    MATCH_INVALID_UTF = 67108864,
    ALT_EXTENDED_CLASS = 134217728,

    NOTBOL = 1,
    NOTEOL = 2,
    NOTEMPTY = 4,
    NOTEMPTY_ATSTART = 8,
    PARTIAL_SOFT = 16,
    PARTIAL_HARD = 32,
    DFA_RESTART = 64,
    DFA_SHORTEST = 128,
    SUBSTITUTE_GLOBAL = 256,
    SUBSTITUTE_EXTENDED = 512,
    SUBSTITUTE_UNSET_EMPTY = 1024,
    SUBSTITUTE_UNKNOWN_UNSET = 2048,
    SUBSTITUTE_OVERFLOW_LENGTH = 4096,
    NO_JIT = 8192,
    COPY_MATCHED_SUBJECT = 16384,
    SUBSTITUTE_LITERAL = 32768,
    SUBSTITUTE_MATCHED = 65536,
    SUBSTITUTE_REPLACEMENT_ONLY = 131072,

    INFO_ALLOPTIONS = 0,
    INFO_ARGOPTIONS = 1,
    INFO_BACKREFMAX = 2,
    INFO_BSR = 3,
    INFO_CAPTURECOUNT = 4,
    INFO_FIRSTCODEUNIT = 5,
    INFO_FIRSTCODETYPE = 6,
    INFO_FIRSTBITMAP = 7,
    INFO_HASCRORLF = 8,
    INFO_JCHANGED = 9,
    INFO_JITSIZE = 10,
    INFO_LASTCODEUNIT = 11,
    INFO_LASTCODETYPE = 12,
    INFO_MATCHEMPTY = 13,
    INFO_MATCHLIMIT = 14,
    INFO_MAXLOOKBEHIND = 15,
    INFO_MINLENGTH = 16,
    INFO_NAMECOUNT = 17,
    INFO_NAMEENTRYSIZE = 18,
    INFO_NAMETABLE = 19,
    INFO_NEWLINE = 20,
    INFO_DEPTHLIMIT = 21,
    INFO_RECURSIONLIMIT = 21,
    INFO_SIZE = 22,
    INFO_HASBACKSLASHC = 23,
    INFO_FRAMESIZE = 24,
    INFO_HEAPLIMIT = 25,
    INFO_EXTRAOPTIONS = 26,
};

local PCRE2_UNSET = Pcre2_Constant.UNSET;
local PCRE2_ZERO_TERMINATED = Pcre2_Constant.ZERO_TERMINATED;
local PCRE2_INFO_NAMECOUNT     = Pcre2_Constant.INFO_NAMECOUNT;
local PCRE2_INFO_NAMEENTRYSIZE = Pcre2_Constant.INFO_NAMEENTRYSIZE;
local PCRE2_INFO_NAMETABLE     = Pcre2_Constant.INFO_NAMETABLE;
local PCRE2_SUBSTITUTE_GLOBAL  = Pcre2_Constant.SUBSTITUTE_GLOBAL;
local PCRE2_ERROR_NOMEMORY     = -48;
local regex_default_options = bit.bor(Pcre2_Constant.UCP, Pcre2_Constant.UTF);
local regex_sub_default_options = bit.bor(
    Pcre2_Constant.SUBSTITUTE_EXTENDED,
    Pcre2_Constant.SUBSTITUTE_UNSET_EMPTY,
    Pcre2_Constant.SUBSTITUTE_OVERFLOW_LENGTH
);
-- 辅助函数：通过反射读取正则内部的 Nametable，并自动把名字注入到 Lua 表中
local function attach_named_captures(code, result)
    local count_ptr = ffi.new(OutU32);
    __api.pcre2.pcre2_pattern_info_8(code, PCRE2_INFO_NAMECOUNT, count_ptr);

    local count = count_ptr[0];
    if count == 0 then return end

    local size_ptr = ffi.new(OutU32);
    __api.pcre2.pcre2_pattern_info_8(code, PCRE2_INFO_NAMEENTRYSIZE, size_ptr);

    local table_ptr = ffi.new(OutU8Ptr);
    __api.pcre2.pcre2_pattern_info_8(code, PCRE2_INFO_NAMETABLE, table_ptr);

    local size = size_ptr[0];
    local nametable = table_ptr[0];

    for i = 0, count - 1 do
        local entry = nametable + i * size;
        -- 前 2 个字节是 16 位的大端整数（表示捕获组数字索引）
        local group_num = bit.lshift(entry[0], 8) + entry[1];
        -- 第 3 个字节开始是 zero-terminated 字符串（组名）
        local group_name = ffi.string(entry + 2);
        result[group_name] = result[group_num];
    end
end
Regex = ffi.metatype("pcre2_regex_t", {
    __new = function(ct, pattern, options)
        local errcode = ffi.new(OutInt);
        local erroffset = ffi.new(OutSize);

        local pat;
        local len;
        if ffi.istype(StrRef, pattern) then
            pat = pattern.ptr;
            len = pattern.len;
        elseif type(pattern) == "string" then
            pat = pattern;
            len = #pattern;
        elseif ffi.istype(char_buf_t, pattern) then
            pat = pattern;
            len = PCRE2_ZERO_TERMINATED;
        else
            error("invalid argument for Regex.new, except StrRef|string|const uint8_t*");
        end
        local code = __api.pcre2.pcre2_compile_8(
            pat,
            len,
            options or regex_default_options,
            errcode,
            erroffset,
            ffi.cast(voidptr_t, 0)
        );

        if code == nil then
            error("PCRE2 compilation failed at offset " .. tonumber(erroffset[0]))
        end
        local match_data = __api.pcre2.pcre2_match_data_create_from_pattern_8(code, ffi.cast(voidptr_t, 0))

        return ffi.new(ct, code, match_data)
    end,
    __gc = function(self)
        if self.code ~= ffi.cast(voidptr_t, 0) then
            __api.pcre2.pcre2_code_free_8(self.code)
        end
        if self.match_data ~= ffi.cast(voidptr_t, 0) then
            __api.pcre2.pcre2_match_data_free_8(self.match_data)
        end
    end,
    __index = {
        Constant = Pcre2_Constant,
        jit_compile = function(self)
            local rc = __api.pcre2.pcre2_jit_compile_8(self.code, 1);
            if rc < 0 then return false, "JIT compilation failed, error code: " .. tonumber(rc) end
            return true;
        end,
        is_match = function(self, subject)
            local sbj;
            local len;
            if ffi.istype(StrRef, subject) then
                sbj = subject.ptr;
                len = subject.len;
            elseif type(subject) == "string" then
                sbj = subject;
                len = #subject;
            elseif ffi.istype(char_buf_t, subject) then
                sbj = subject;
                len = PCRE2_ZERO_TERMINATED;
            else
                error("invalid argument for Regex.is_match, except StrRef|string|const uint8_t*");
            end

            local rc = __api.pcre2.pcre2_match_8(
                self.code,
                sbj,
                len,
                0, -- startoffset
                0, -- options
                self.match_data,
                ffi.cast(voidptr_t, 0)
            );
            return rc >= 0;
        end,
        find = function(self, subject, start_pos)
            local sbj;
            local len;
            local get_slice;
            if ffi.istype(StrRef, subject) then
                sbj = subject.ptr;
                len = subject.len;
                get_slice = function (start_idx, end_idx) return subject:slice(start_idx, end_idx) end;
            elseif type(subject) == "string" then
                sbj = subject;
                len = #subject;
                get_slice = function (start_idx, end_idx) return string.sub(subject, start_idx+1, end_idx) end;
            elseif ffi.istype(char_buf_t, subject) then
                sbj = subject;
                len = PCRE2_ZERO_TERMINATED;
                get_slice = function (start_idx, end_idx) return ffi.new(StrRef, subject+start_idx, end_idx-start_idx) end;
            else
                error("invalid argument for Regex.find, except StrRef|string|const uint8_t*");
            end
            -- 总是使用 0-started，即便是 string
            start_pos = start_pos or 0;
            if start_pos < 0 then start_pos = 0 end
            if start_pos >= len then return nil end

            local rc = __api.pcre2.pcre2_match_8(
                self.code, sbj, len, start_pos, 0, self.match_data, ffi.cast(voidptr_t, 0)
            );

            if rc >= 0 then
                local ovector = __api.pcre2.pcre2_get_ovector_pointer_8(self.match_data);
                return get_slice(tonumber(ovector[0]), tonumber(ovector[1]));
            end

            return nil
        end,
        captures = function(self, subject, start_pos)
            local sbj;
            local len;
            local get_slice;
            if ffi.istype(StrRef, subject) then
                sbj = subject.ptr;
                len = subject.len;
                get_slice = function (start_idx, end_idx) return subject:slice(start_idx, end_idx) end;
            elseif type(subject) == "string" then
                sbj = subject;
                len = #subject;
                get_slice = function (start_idx, end_idx) return string.sub(subject, start_idx+1, end_idx) end;
            elseif ffi.istype(char_buf_t, subject) then
                sbj = subject;
                len = PCRE2_ZERO_TERMINATED;
                get_slice = function (start_idx, end_idx) return ffi.new(StrRef, subject+start_idx, end_idx-start_idx) end;
            else
                error("invalid argument for Regex.captures, except StrRef|string|const uint8_t*");
            end

            start_pos = start_pos or 0;
            if start_pos < 0 then start_pos = 0 end
            if start_pos >= len then return nil end

            local rc = __api.pcre2.pcre2_match_8(
                self.code, sbj, len, start_pos, 0, self.match_data, ffi.cast(voidptr_t, 0)
            );
            if rc < 0 then return nil end

            local ovector = __api.pcre2.pcre2_get_ovector_pointer_8(self.match_data);
            local result = {};

            -- rc 是命中的捕获组数量，包含整个完整匹配
            -- i=0 是完整匹配，i>0 是各个括号的捕获组
            for i = 0, tonumber(rc) - 1 do
                local s = ovector[2 * i];
                local e = ovector[2 * i + 1];

                -- 如果是未匹配的可选捕获组（如 (a)? 没有命中），偏移量会返回 PCRE2_UNSET
                if s == PCRE2_UNSET then
                    result[i] = false;
                else
                    result[i] = get_slice(tonumber(s), tonumber(e));
                end
            end

            attach_named_captures(self.code, result);
            return result;
        end,
        gmatch = function(self, subject, start_pos)
            local sbj;
            local len;
            local get_slice;
            local get_bytes;
            if ffi.istype(StrRef, subject) then
                sbj = subject.ptr;
                len = subject.len;
                get_slice = function (start_idx, end_idx) return subject:slice(start_idx, end_idx) end;
                get_bytes = function (idx) return tonumber((subject.ptr + idx)[0]) end;
            elseif type(subject) == "string" then
                sbj = subject;
                len = #subject;
                get_slice = function (start_idx, end_idx) return string.sub(subject, start_idx+1, end_idx) end;
                get_bytes = function(idx) return string.byte(subject, idx+1) end;
            elseif ffi.istype(char_buf_t, subject) then
                sbj = subject;
                len = PCRE2_ZERO_TERMINATED;
                get_slice = function (start_idx, end_idx) return ffi.new(StrRef, subject+start_idx, end_idx-start_idx) end;
                get_bytes = function (idx) return tonumber((subject.ptr + idx)[0]) end;
            else
                error("invalid argument for Regex.gmatch, except StrRef|string|const uint8_t*");
            end

            start_pos = start_pos or 0;
            if start_pos < 0 then start_pos = 0 end
            local offset = start_pos;

            return function()
                if offset > len then return nil end

                local rc = __api.pcre2.pcre2_match_8(
                    self.code, sbj, len, offset, 0, self.match_data, ffi.cast(voidptr_t, 0)
                );
                if rc < 0 then return nil end

                local ovector = __api.pcre2.pcre2_get_ovector_pointer_8(self.match_data);
                local s = tonumber(ovector[0]);
                local e = tonumber(ovector[1]);

                -- 防止零长度匹配导致死循环
                if s == e then
                    local b = get_bytes(e);
                    local skip = 1;
                    if b then
                        if b >= 0xC0 and b < 0xE0 then skip = 2
                        elseif b >= 0xE0 and b < 0xF0 then skip = 3
                        elseif b >= 0xF0 then skip = 4 end
                    end
                    offset = e + skip;
                else
                    offset = e;
                end

                local result = {}
                for i = 0, tonumber(rc) - 1 do
                    local cs = ovector[2 * i];
                    local ce = ovector[2 * i + 1];
                    if cs == PCRE2_UNSET then
                        result[i] = false;
                    else
                        result[i] = get_slice(tonumber(cs), tonumber(ce));
                    end
                end

                attach_named_captures(self.code, result);
                return result;
            end
        end,
        gsub = function(self, subject, replacement, options, once)
            if not ffi.istype(Regex, self) then
                self = Regex(self);
            end

            options = options or regex_sub_default_options;
            if once then
                options = bit.band(options, bit.bnot(PCRE2_SUBSTITUTE_GLOBAL));
            else
                options = bit.bor(options, PCRE2_SUBSTITUTE_GLOBAL);
            end

            local sbj, sbj_len;
            if ffi.istype(StrRef, subject) then
                sbj, sbj_len = subject.ptr, subject.len;
            elseif type(subject) == "string" then
                sbj, sbj_len = subject, #subject;
            elseif ffi.istype(char_buf_t, subject) then
                sbj, sbj_len = subject, PCRE2_ZERO_TERMINATED;
            else
                error("invalid argument for Regex.is_match, except StrRef|string|const uint8_t*");
            end
            local rep, rep_len;
            if ffi.istype(StrRef, replacement) then
                rep, rep_len = replacement.ptr, replacement.len;
            elseif type(replacement) == "string" then
                rep, rep_len = replacement, #replacement;
            elseif ffi.istype(char_buf_t, replacement) then
                rep, rep_len = replacement, PCRE2_ZERO_TERMINATED;
            else
                error("invalid argument for Regex.is_match, except StrRef|string|const uint8_t*");
            end

            -- 预估输出缓冲区大小（被替换字符串长度 + 替换文本长度 * 2 + 128余量）
            local out_size = sbj_len + rep_len * 2 + 128;
            local out_len = ffi.new(OutSize, out_size);
            local out_buf = ffi.new(U8VLA, out_size);

            local rc = __api.pcre2.pcre2_substitute_8(
                self.code,
                sbj, sbj_len,
                0,
                options,
                self.match_data,
                nil,
                rep, rep_len,
                out_buf,
                out_len
            );

            if rc == PCRE2_ERROR_NOMEMORY then
                -- 因为前面传了 OVERFLOW_LENGTH，out_len[0] 现在刚好等于实际所需的确切长度 (含 \0)
                out_size = tonumber(out_len[0]);
                out_buf = ffi.new(U8VLA, out_size);

                rc = __api.pcre2.pcre2_substitute_8(
                    self.code, sbj, sbj_len, 0, options,
                    self.match_data, nil, rep, rep_len,
                    out_buf, out_len
                );
            end

            if rc < 0 then
                local err_buf = ffi.new(U8VLA, 128);
                local err_len = __api.pcre2.pcre2_get_error_message_8(rc, err_buf, 128);
                error("PCRE2 substitution failed with error code " .. tonumber(rc) .. ": " .. ffi.string(err_buf, err_len));
            end

            return ffi.string(out_buf, out_len[0]), tonumber(rc);
        end
    }
});

local sub = function (s, i, j)
    if not j then
        j = #s;
    end
    if type(s) == "string" then
        return string.sub(s, i, j);
    elseif ffi.istype(StrRef, s) then
        return s:slice(i - 1, j);
    else
        return nil;
    end
end;
local starts_with = function (s, target)
    local end_idx = #target;
    if type(s) == "string" then
        return sub(s, 1, end_idx) == target;
    elseif ffi.istype(StrRef, s) then
        return s:slice(0, end_idx) == target;
    else
        return nil;
    end
end;

local latin_group = {
    [0x41] = 'A', [0x42] = 'B', [0x43] = 'C', [0x44] = 'D', [0x45] = 'E', [0x46] = 'F', [0x47] = 'G', [0x48] = 'H',
    [0x49] = 'I', [0x4A] = 'J', [0x4B] = 'K', [0x4C] = 'L', [0x4D] = 'M', [0x4E] = 'N', [0x4F] = 'O', [0x50] = 'P',
    [0x51] = 'Q', [0x52] = 'R', [0x53] = 'S', [0x54] = 'T', [0x55] = 'U', [0x56] = 'V', [0x57] = 'W', [0x58] = 'X',
    [0x59] = 'Y', [0x5A] = 'Z',
    [0x61] = 'A', [0x62] = 'B', [0x63] = 'C', [0x64] = 'D', [0x65] = 'E', [0x66] = 'F', [0x67] = 'G', [0x68] = 'H',
    [0x69] = 'I', [0x6A] = 'J', [0x6B] = 'K', [0x6C] = 'L', [0x6D] = 'M', [0x6E] = 'N', [0x6F] = 'O', [0x70] = 'P',
    [0x71] = 'Q', [0x72] = 'R', [0x73] = 'S', [0x74] = 'T', [0x75] = 'U', [0x76] = 'V', [0x77] = 'W', [0x78] = 'X',
    [0x79] = 'Y', [0x7A] = 'Z',
};
local bushou_to_chars = cindex_table.BuShouToChars;
local special_char = {
    [0x3007] = { -- '〇'
        Mandarin = "ling2", -- ling2
        BiHua = "BiHua1",   -- 1 画
        BuShou = "BuShou5", -- 认为是“⼄”部
        KangXiNumber = 5,   -- 康熙部首第 5 部
        BiShun = "5",       -- 折
        AdditionalStrokes = 0,
    }
}
setmetatable(special_char, {
    __index = {
        mandarin = function (self, chr)
            local t = rawget(self, chr);
            if t and t.Mandarin and #t.Mandarin > 0 then
                local man = t.Mandarin;
                local last_n = string.byte(man, #man);
                if last_n >= 0x30 and last_n <= 0x39 then
                    return string.sub(man, 1, #man - 1), (last_n - 0x30);
                end
            end
            return nil, nil;
        end,
        total_strokes = function (self, chr)
            local t = rawget(self, chr);
            if t and t.BiHua and string.sub(t.BiHua, 1, 5) == "BiHua" then
                return tonumber(string.sub(t.BiHua, 6));
            end
            return nil;
        end,
        ucd_additional_strokes = function (self, chr)
            local t = rawget(self, chr);
            if t and t.AdditionalStrokes then
                return t.AdditionalStrokes;
            else
                return nil;
            end
        end,
        kangxi_radical_number = function (self, chr)
            local t = rawget(self, chr);
            if t and t.KangXiNumber then
                return t.KangXiNumber;
            else
                return nil;
            end
        end,
        kangxi_radical_ideography_slot = function (self, chr)
            local t = rawget(self, chr);
            if t and t.BuShou then
                return bushou_to_chars[t.BuShou][1];
            else
                return nil;
            end
        end,
        han_ordered_strokes = function (self, chr)
            local t = rawget(self, chr);
            if t and t.BiShun then
                return t.BiShun;
            else
                return nil;
            end
        end,
    }
});
local bihua_to_group = cindex_table.BiHuaToGroup;
local bushou_to_group = cindex_table.BuShouToGroup;
CIndex = {
    Util = {},
    Ideographic = {},
    Entry = {},
}
CIndex.Util.is_ctype = ffi.istype;
local table_to_json = __c_function_api.table_to_json;
CIndex.Util.table_to_json_string = table_to_json;
local value_from_json = __c_function_api.value_from_json;
CIndex.Util.value_from_json_string = value_from_json;
CIndex.Ideographic.group_by_pinyin = function (chr)
    local sp = special_char[tonumber(chr.inner)];
    if sp then
        local g = sp.Mandarin;
        if g and #g > 0 then return latin_group[string.byte(g, 1)] end
    end
    local cjk_info = chr:han_info();
    if not (cjk_info == nil) then
        local mandarin = cjk_info:mandarin();
        if mandarin and not mandarin:is_empty() then
            local g = mandarin:head();
            if g then return latin_group[g] end
        end
    end
    -- 没有拼音的，一律为符号！
    return "Symbols";
end;
CIndex.Ideographic.group_by_bihua = function (chr)
    local sp = special_char[tonumber(chr.inner)];
    if sp then
        local g = sp.BiHua;
        if g then return g end
    end
    local cjk_info = chr:han_info();
    if not (cjk_info == nil) then
        local sn = tonumber(cjk_info.glyph_total_strokes);
        if sn > 0 then
            if sn <= 64 then
                return bihua_to_group[sn];
            else
                return bihua_to_group[64]; -- 最多 64 画
            end
        end
    end
    -- 没有笔画的，一律为符号！
    return "Symbols";
end;
CIndex.Ideographic.group_by_bushou = function (chr)
    local sp = special_char[tonumber(chr.inner)];
    if sp then
        local g = sp.BuShou;
        if g then return g end
    end
    local cjk_info = chr:han_info();
    if not (cjk_info == nil) then
        local ideo = tonumber(cjk_info:kx_ideography_char().inner);
        if ideo then
            return bushou_to_group[ideo];
        end
    end
    -- 没有部首的，一律为符号！
    return "Symbols";
end;
CIndex.Entry.compare_by_default = function (lhs, rhs)
    local len_lhs = lhs:levels_count();
    local len_rhs = rhs:levels_count();
    if len_lhs == 0 then
        if len_rhs == 0 then return false end
        return true;
    elseif len_rhs == 0 then
        return false;
    end
    local l_key, r_key;
    local cmp_result;
    for i = 0, math.min(len_lhs, len_rhs) - 1 do
        l_key = lhs:key_n(i);
        r_key = rhs:key_n(i);
        cmp_result = l_key:cmp_ignore_case(r_key);
        if cmp_result == 0 then
            cmp_result = l_key:cmp(r_key);
            if cmp_result == 0 then
                -- continue
            elseif cmp_result == 1 then
                return false;
            else -- nil or -1
                return true;
            end
        elseif cmp_result == 1 then
            return false;
        else -- nil or -1
            return true;
        end
    end
    return len_lhs <= len_rhs;
end;
local char_cmp_case_fold = function (l, r)
    return Char(l):simple_fold():cmp(Char(r):simple_fold());
end;
-- 使用读音排序时，没有读音数据的字符（包括生僻字）一律排在有读音的字符之前，
-- 汉字按其最常用读音比较，读音相同汉字的按 Unicode 编码排序。
CIndex.Entry.compare_by_pinyin = function (lhs, rhs)
    local len_lhs = lhs:levels_count();
    local len_rhs = rhs:levels_count();
    if len_lhs == 0 then
        if len_rhs == 0 then return false end
        return true;
    elseif len_rhs == 0 then
        return false;
    end
    local l_key, r_key;
    for i = 0, math.min(len_lhs, len_rhs) - 1 do
        l_key = lhs:key_n(i);
        r_key = rhs:key_n(i);
        local l_key_chars = l_key:chars();
        local r_key_chars = r_key:chars();
        while true do
            -- l_char, r_char : nil|number
            local l_char = l_key_chars();
            local r_char = r_key_chars();
            if l_char and r_char then
                if l_char == r_char then
                    goto continue;
                end
            elseif l_char then
                -- lhs 有剩余的字符，rhs 没有，说明 lhs 更长，排在后面
                return false;
            elseif r_char then
                return true;
            else
                break;
            end

            local l_info = CjkInfo(l_char);
            local r_info = CjkInfo(r_char);
            local l_is_cjk = (l_info ~= nil) or (special_char[l_char] ~= nil);
            local r_is_cjk = (r_info ~= nil) or (special_char[r_char] ~= nil);

            if l_is_cjk and r_is_cjk then
                -- 两个都是汉字：按拼音比较
                local l_man, l_tone = special_char:mandarin(l_char);
                if not l_man then
                    l_man, l_tone = l_info:mandarin();
                end
                local r_man, r_tone = special_char:mandarin(r_char);
                if not r_man then
                    r_man, r_tone = r_info:mandarin();
                end

                if l_man and not r_man then
                    -- l 有读音，r 没有，r 排在前面
                    return false;
                elseif not l_man and r_man then
                    return true;
                elseif not l_man and not r_man then
                    -- 两个都没有读音，按 Unicode 排序
                    local r_caseless = char_cmp_case_fold(l_char, r_char);
                    if r_caseless ~= 0 then
                        return r_caseless < 0;
                    end
                elseif l_man == r_man then
                    if l_tone ~= r_tone then
                        return l_tone < r_tone;
                    end
                    -- 拼音和声调都相同，继续比较下一个字符
                else
                    return l_man < r_man;
                end

            elseif not l_is_cjk and not r_is_cjk then
                -- 两个都不是汉字：直接按 Unicode 码点
                local r_caseless = char_cmp_case_fold(l_char, r_char);
                if r_caseless ~= 0 then
                    return r_caseless < 0;
                end
            else
                -- 混合：非汉字排在前面
                if not l_is_cjk then  -- l 非汉字，r 汉字
                    return true;
                else                  -- l 汉字，r 非汉字
                    return false;
                end
            end
            ::continue::
        end
        -- 两个 key 相等，再次判断 level 是否相等
        local level_cmp = lhs:level_n(i):cmp(rhs:level_n(i));
        if level_cmp ~= 0 then
            return level_cmp < 0;
        end
    end
    return len_lhs < len_rhs;
end;
-- 使用笔画数和笔顺排序时，笔画数小的排在前面，笔画数相同的，按横、竖、撇、点（捺）、折的顺序逐笔画比较，
-- 仍然相同的按 Unicode 编码排序；生僻汉字没有笔顺信息的，排在同笔画数有笔顺的字后面。
CIndex.Entry.compare_by_bihua = function (lhs, rhs)
    local len_lhs = lhs:levels_count();
    local len_rhs = rhs:levels_count();
    if len_lhs == 0 then
        if len_rhs == 0 then return false end
        return true;
    elseif len_rhs == 0 then
        return false;
    end
    local l_key, r_key;
    for i = 0, math.min(len_lhs, len_rhs) - 1 do
        l_key = lhs:key_n(i);
        r_key = rhs:key_n(i);
        local l_key_chars = l_key:chars();
        local r_key_chars = r_key:chars();
        while true do
            -- l_char, r_char : nil|number
            local l_char = l_key_chars();
            local r_char = r_key_chars();
            if l_char and r_char then
                if l_char == r_char then
                    goto continue;
                end
            elseif l_char then
                -- lhs 有剩余的字符，rhs 没有，说明 lhs 更长，排在后面
                return false;
            elseif r_char then
                return true;
            else
                break;
            end

            local l_info = CjkInfo(l_char);
            local r_info = CjkInfo(r_char);
            local l_is_cjk = (l_info ~= nil) or (special_char[l_char] ~= nil);
            local r_is_cjk = (r_info ~= nil) or (special_char[r_char] ~= nil);

            if l_is_cjk and r_is_cjk then
                -- 两个都是汉字，按笔画数和笔顺比较
                local l_total_strokes = special_char:total_strokes(l_char) or l_info:total_strokes();
                local r_total_strokes = special_char:total_strokes(r_char) or r_info:total_strokes();
                if l_total_strokes and r_total_strokes then
                    if l_total_strokes == r_total_strokes then
                        -- 笔画数相同，按笔顺比较
                        local l_bishun = special_char:han_ordered_strokes(l_char) or Char(l_char):han_ordered_strokes();
                        local r_bishun = special_char:han_ordered_strokes(r_char) or Char(r_char):han_ordered_strokes();
                        if l_bishun and r_bishun then
                            local cmp_res;
                            if ffi.istype(OrderedStrokes, l_bishun) and ffi.istype(OrderedStrokes, r_bishun) then
                                cmp_res = OrderedStrokes.cmp(l_bishun, r_bishun);
                            elseif type(l_bishun) == "string" and type(r_bishun) == "string" then
                                if l_bishun < r_bishun then
                                    cmp_res = -1;
                                elseif l_bishun > r_bishun then
                                    cmp_res = 1;
                                else
                                    cmp_res = 0;
                                end
                            else
                                cmp_res = OrderedStrokes.cmp_with_str(l_bishun, r_bishun);
                            end
                            if cmp_res == 0 then
                                -- 笔顺仍然相同的，按 Unicode 比较
                                cmp_res = char_cmp_case_fold(l_char, r_char);
                                if cmp_res ~= 0 then
                                    return cmp_res < 0;
                                end
                            else
                                return cmp_res < 0;
                            end
                        elseif l_bishun and not r_bishun then
                            -- l 有笔顺信息，r 没有，r 排在后面
                            return true;
                        elseif not l_bishun and r_bishun then
                            return false;
                        else
                            -- 两个都没有，按 Unicode 排序，此处 l 与 r 不可能相等
                            local cmp_res = char_cmp_case_fold(l_char, r_char);
                            if cmp_res ~= 0 then
                                return cmp_res < 0;
                            end
                        end
                    else
                        return l_total_strokes < r_total_strokes;
                    end
                else
                    local nil_char;
                    if not l_total_strokes then
                        nil_char = string.format("%X", l_char);
                    else
                        nil_char = string.format("%X", r_char);
                    end
                    -- 凡是 CJK 表意字或兼容区文字，必有 total_strokes 的信息
                    error("unreachable, missing total strokes for U+" .. nil_char);
                end

            elseif not l_is_cjk and not r_is_cjk then
                -- 两个都不是汉字：直接按 Unicode 码点
                local cmp_res = char_cmp_case_fold(l_char, r_char);
                if cmp_res ~= 0 then
                    return cmp_res < 0;
                end
            else
                -- 混合：非汉字排在前面
                if not l_is_cjk then  -- l 非汉字，r 汉字
                    return true;
                else                  -- l 汉字，r 非汉字
                    return false;
                end
            end
            ::continue::
        end
        -- 两个 key 相等，再次判断 level 是否相等
        local level_cmp = lhs:level_n(i):cmp(rhs:level_n(i));
        if level_cmp ~= 0 then
            return level_cmp < 0;
        end
    end
    return len_lhs < len_rhs;
end;
-- 使用部首和除部首笔画数排序时，部首按康熙字典214 部首顺序排列，
-- 部首和笔画数相同的按 Unicode 编码排序。
CIndex.Entry.compare_by_bushou = function (lhs, rhs)
    local len_lhs = lhs:levels_count();
    local len_rhs = rhs:levels_count();
    if len_lhs == 0 then
        if len_rhs == 0 then return false end
        return true;
    elseif len_rhs == 0 then
        return false;
    end
    local l_key, r_key;
    for i = 0, math.min(len_lhs, len_rhs) - 1 do
        l_key = lhs:key_n(i);
        r_key = rhs:key_n(i);
        local l_key_chars = l_key:chars();
        local r_key_chars = r_key:chars();
        while true do
            -- l_char, r_char : nil|number
            local l_char = l_key_chars();
            local r_char = r_key_chars();
            if l_char and r_char then
                if l_char == r_char then
                    goto continue;
                end
            elseif l_char then
                -- lhs 有剩余的字符，rhs 没有，说明 lhs 更长，排在后面
                return false;
            elseif r_char then
                return true;
            else
                break;
            end

            local l_info = CjkInfo(l_char);
            local r_info = CjkInfo(r_char);
            local l_is_cjk = (l_info ~= nil) or (special_char[l_char] ~= nil);
            local r_is_cjk = (r_info ~= nil) or (special_char[r_char] ~= nil);

            if l_is_cjk and r_is_cjk then
                -- 两个都是汉字，按部首和除部首外的笔画数排序
                local l_radical = special_char:kangxi_radical_number(l_char) or l_info:kangxi_radical_number();
                local r_radical = special_char:kangxi_radical_number(r_char) or r_info:kangxi_radical_number();
                if l_radical and r_radical then
                    if l_radical == r_radical then
                        -- 部首相同，按除部首外的笔画数比较
                        local l_addi = special_char:ucd_additional_strokes(l_char) or l_info:ucd_additional_strokes();
                        local r_addi = special_char:ucd_additional_strokes(r_char) or r_info:ucd_additional_strokes();
                        if l_addi and r_addi then
                            if l_addi == r_addi then
                                -- 仍然相同的，按 Unicode 比较
                                local cmp_res = char_cmp_case_fold(l_char, r_char);
                                if cmp_res ~= 0 then
                                    return cmp_res < 0;
                                end
                            else
                                return l_addi < r_addi;
                            end
                        else
                            -- 凡是 CJK 表意字或兼容区文字，必有除部首外笔画数的信息
                            local nil_char;
                            if not l_radical then
                                nil_char = string.format("%X", l_char);
                            else
                                nil_char = string.format("%X", r_char);
                            end
                            -- 凡是 CJK 表意字或兼容区文字，必有部首的信息
                            error("unreachable, missing additional strokes for U+" .. nil_char);
                        end
                    else
                        return l_radical < r_radical;
                    end
                else
                    local nil_char;
                    if not l_radical then
                        nil_char = string.format("%X", l_char);
                    else
                        nil_char = string.format("%X", r_char);
                    end
                    -- 凡是 CJK 表意字或兼容区文字，必有部首的信息
                    error("unreachable, missing radical for U+" .. nil_char);
                end

            elseif not l_is_cjk and not r_is_cjk then
                -- 两个都不是汉字：直接按 Unicode 码点
                local cmp_res = char_cmp_case_fold(l_char, r_char);
                if cmp_res ~= 0 then
                    return cmp_res < 0;
                end
            else
                -- 混合：非汉字排在前面
                if not l_is_cjk then  -- l 非汉字，r 汉字
                    return true;
                else                  -- l 汉字，r 非汉字
                    return false;
                end
            end
            ::continue::
        end
        -- 两个 key 相等，再次判断 level 是否相等
        local level_cmp = lhs:level_n(i):cmp(rhs:level_n(i));
        if level_cmp ~= 0 then
            return level_cmp < 0;
        end
    end
    return len_lhs < len_rhs;
end;

local function deep_copy(o, lookup_table)
    lookup_table = lookup_table or {};
    if type(o) ~= "table" then
        return o;
    end
    if lookup_table[o] then
        return lookup_table[o];
    end
    local new_table = {};
    lookup_table[o] = new_table;
    for key, value in pairs(o) do
        new_table[deep_copy(key, lookup_table)] = deep_copy(value, lookup_table);
    end
    local mt = getmetatable(o);
    if mt then
        setmetatable(new_table, deep_copy(mt, lookup_table));
    end
    return new_table;
end

local safe_env = setmetatable({}, { __index = _G });
local preload = {};
for k, v in pairs(package.preload) do
    if string.sub(k, 1, 3) ~= "jit" then
        preload[k] = v;
    end
end
safe_env.platform = {
    version = jit.version,
    version_num = jit.version_num,
    os = jit.os,
    arch = jit.arch,
    status = jit.status,
};
safe_env.os.remove = nil;
safe_env.os.execute = nil;
safe_env.os.rename = nil;
safe_env.io = nil;
safe_env.debug = nil;
local loaded = { bit = bit, cindex = safe_env.CIndex };
safe_env.package = { preload = preload, loaded = loaded, cpath = nil, path = nil };
safe_env.require = function (name)
    local t = loaded[name];
    if t then
        return t;
    end
    local f = preload[name];
    if not f then
        error("unable to load \"" .. name .. "\" package");
    else
        return f(name);
    end
end;
safe_env.ffi = nil;
safe_env.jit = nil;
safe_env.dofile = nil;
safe_env.loadfile = nil;
safe_env.__api = __api;
safe_env._G = safe_env;

local function load_user_script(script_path)
    if not script_path then return false, nil end

    local chunk, err = loadfile(script_path);
    if not chunk then
        error(err);
        return false, nil;
    end

    setfenv(chunk, safe_env);

    local ok, result = pcall(chunk);
    if ok then
        return true, result;
    else
        error("User script \"" .. script_path .. "\" error: " .. tostring(result));
        return false, nil
    end
end

local has_custom, user_script = load_user_script(script)


local detect_groups_with = function (group_func, start_ptr, size_of, len, result_ptr)
    start_ptr = ffi.cast("uint8_t*", start_ptr);
    result_ptr = ffi.cast("uint32_t*", result_ptr);
    local result = {};
    if len == 0 then
        return result;
    end

    local entry = ffi.new(MergedEntryRef);
    for i = 0, len-1 do
        entry.inner = ffi.cast(voidptr_t, start_ptr + i * size_of);
        local key = tostring(group_func(entry));
        if result[key] then
            -- 如果已经存在，则返回整数索引
            result_ptr[i] = result[key];
        else
            -- 否则，由于 Rust 中使用 0-started，不存在时，值应该为 0，
            -- 同时我们在其中附加上这个 key，这是 1-started。偏移值总是 1。
            result_ptr[i] = #result;
            result[key] = #result;
            table.insert(result, key);
        end
    end

    return result;
end

if has_custom and type(user_script) == "table" then
    local gen_func = function(raw, key, func)
        if type(user_script[raw]) == "function" then
            -- _G[raw] = user_script[raw];
            error("The function '" .. raw .. "' must not be provided!");
        elseif type(_G[raw]) == "function" then
            -- already defined _G[raw], by pass
        elseif type(user_script[key]) == "function" then
            _G[raw] = func;
        end
    end;
    local get_value = function (key, default_value, value_type)
        if value_type ~= nil then
            if type(user_script[key]) == value_type then
                _G[key] = user_script[key];
            elseif type(user_script[key]) == "nil" then
                _G[key] = default_value;
            else
                error("expect " .. value_type .. " for \'" .. key .. "\'");
            end
        elseif type(user_script[key]) == "nil" then
            _G[key] = default_value;
        else
            _G[key] = user_script[key];
        end
    end;

    gen_func("__cindex_read_indices", "parse_indices_string", function (file_path, s, is_ikv, vec)
        local parse_indices_string = user_script.parse_indices_string;
        local lines = StrRef.from_ptr(ffi.cast(voidptr_t, s));
        local arr = ffi.new(IndexEntryArray, ffi.cast(voidptr_t, vec));
        arr:extend_from_iterator(function() return parse_indices_string(file_path, lines, is_ikv) end);
    end);
    gen_func("__cindex_read_indices", "parse_index_line", function (file_path, s, is_ikv, vec)
        local parse_index_line = user_script.parse_index_line;
        local line_func = StrRef.from_ptr(ffi.cast(voidptr_t, s)):lines();
        local arr = ffi.new(IndexEntryArray, ffi.cast(voidptr_t, vec));
        local iterator_init = function ()
            return function () local line = line_func();
                if line then return parse_index_line(line) else return line end
            end;
        end;
        if file_path then Logger.info("Reading input '" .. file_path .. "'.") end
        arr:extend_from_iterator(iterator_init);
    end);
    gen_func("__cindex_reading_empty", "empty_indices_callout", function (vec)
        user_script.empty_indices_callout(ffi.new(IndexEntryArray, ffi.cast(voidptr_t, vec)));
    end);
    gen_func("__cindex_page_precedence", "page_precedence", function (prec)
        if type(prec) == "function" then
            return prec();
        else
            return prec;
        end
    end);
    gen_func("__cindex_detect_groups", "group_detect", function (start_ptr, size_of, len, result_ptr)
        local group_func = user_script.group_detect;
        return detect_groups_with(group_func, start_ptr, size_of, len, result_ptr);
    end);
    gen_func("__cindex_sort_groups", "group_compare", function (groups_table, counts_ptr)
        counts_ptr = ffi.cast("const size_t*", ffi.cast(voidptr_t, counts_ptr));
        local group_compare = user_script.group_compare;
        local counts_table = {};
        for i, v in ipairs(groups_table) do
            counts_table[v] = tonumber(counts_ptr[i-1]);
        end
        table.sort(groups_table, function (a, b)
            return group_compare(a, b, counts_table[a], counts_table[b]);
        end);
    end);
    gen_func("__cindex_sort_entries", "entry_compare", function (target_table)
        local cmp_func = user_script.entry_compare;
        for g, v in pairs(target_table) do
            local real_cmp = cmp_func;
            if user_script["entry_compare_" .. g] then
                real_cmp = user_script["entry_compare_" .. g];
            elseif starts_with(g, "BiHua") and user_script["entry_compare_BiHua"] then
                real_cmp = user_script["entry_compare_BiHua"];
            elseif starts_with(g, "BuShou") and user_script["entry_compare_BuShou"] then
                real_cmp = user_script["entry_compare_BuShou"];
            end
            table.sort(v, function (l, r)
                l = ffi.new(MergedEntryRef, ffi.cast(voidptr_t, l));
                r = ffi.new(MergedEntryRef, ffi.cast(voidptr_t, r));
                local res = real_cmp(l, r, g);
                if res then
                    if res <= 0 then return true else return false end
                else
                    return false;
                end
            end);
        end
    end);
    get_value("output_style", {}, "table");
elseif has_custom and type(user_script) ~= "nil" then
    error("invalid user script: " .. tostring(script));
end

_G.__cindex_read_indices = _G.__cindex_read_indices or function(file_path, s, is_ikv, vec)
    if file_path then Logger.info("Reading input '" .. file_path .. "'.") end
    if is_ikv then
        __api.ikv_input_read_all(__api.ist_input, ffi.cast(voidptr_t, vec), ffi.cast(voidptr_t, s));
    else
        __api.idx_input_read_all(__api.ist_input, ffi.cast(voidptr_t, vec), ffi.cast(voidptr_t, s));
    end
end;
_G.__cindex_reading_empty = _G.__cindex_reading_empty or function (vec) end;
_G.__cindex_page_precedence = _G.__cindex_page_precedence or function()
    return ((not IstOutputStyle:is_nil()) and tostring(IstOutputStyle:page_precedence())) or nil;
end;

local detect_key_func = function(ideo_group_func)
    return function (entry)
        local key_0 = entry:key_n(0);
        if key_0 and not key_0:is_empty() then
            local first_char = Char(key_0:head());
            if first_char:is_ideographic() then
                -- 中文表意字
                return ideo_group_func(first_char);
            elseif first_char.inner >= 0x41 and first_char.inner <= 0x5A then
                -- ASCII Latin uppercase
                return latin_group[tonumber(first_char.inner)];
            elseif first_char.inner >= 0x61 and first_char.inner <= 0x7A then
                -- ASCII Latin lowercase
                return latin_group[tonumber(first_char.inner)];
            elseif first_char:is_numeric() then
                local all_num = true;
                for c in key_0:chars() do
                    if not Char(c):is_numeric() then
                        all_num = false;
                        break;
                    end
                end
                if all_num then return "Numbers" else return "Symbols" end
            else
                return "Symbols"; -- 1
            end
        else
            return "nil"; -- 0
        end
    end;
end;
_G.__cindex_detect_groups = _G.__cindex_detect_groups or function (start_ptr, size_of, len, result_ptr)
    local ideo_group_func;
    if not CliOptions.z then
        ideo_group_func = CIndex.Ideographic.group_by_pinyin;
    else
        local sort_option = StrRef.unsafe_new(CliOptions.z):split(",")();
        if sort_option == "pinyin" or sort_option == "reading" then
            ideo_group_func = CIndex.Ideographic.group_by_pinyin;
        elseif sort_option == "bihua" or sort_option == "stroke" then
            ideo_group_func = CIndex.Ideographic.group_by_bihua;
        elseif sort_option == "bushou" or sort_option == "radical" then
            ideo_group_func = CIndex.Ideographic.group_by_bushou;
        else
            error("unknown sort option: " .. tostring(sort_option));
        end
    end
    return detect_groups_with(detect_key_func(ideo_group_func), start_ptr, size_of, len, result_ptr);
end;

_G.__cindex_sort_groups = _G.__cindex_sort_groups or function (groups_table, counts_ptr)
    -- counts_ptr = ffi.cast("const size_t*", ffi.cast(voidptr_t, counts_ptr));
    local group_order = cindex_table.GroupOrder;
    table.sort(groups_table, function (a, b)
        a = group_order[a];
        b = group_order[b];
        if a and b then return a < b;
        else return a == nil or b == nil;
        end
    end);
end;

_G.__cindex_sort_entries = _G.__cindex_sort_entries or function (target_table)
    local compare_by_default = CIndex.Entry.compare_by_default;
    local compare_by_pinyin = CIndex.Entry.compare_by_pinyin;
    local compare_by_bihua = CIndex.Entry.compare_by_bihua;
    local compare_by_bushou = CIndex.Entry.compare_by_bushou;
    local plain_cmp = function (lhs, rhs)
        return compare_by_default(
            ffi.new(MergedEntryRef, ffi.cast(voidptr_t, lhs)),
            ffi.new(MergedEntryRef, ffi.cast(voidptr_t, rhs))
        );
    end;
    local cmp_symbols = function (lhs, rhs)
        return compare_by_pinyin(
            ffi.new(MergedEntryRef, ffi.cast(voidptr_t, lhs)),
            ffi.new(MergedEntryRef, ffi.cast(voidptr_t, rhs))
        );
    end;
    if CliOptions.z then
        local sort_option = StrRef.unsafe_new(CliOptions.z):split(",")();
        if sort_option == "pinyin" or sort_option == "reading" then
        elseif sort_option == "bihua" or sort_option == "stroke" then
            cmp_symbols = function (lhs, rhs)
                return compare_by_bihua(
                    ffi.new(MergedEntryRef, ffi.cast(voidptr_t, lhs)),
                    ffi.new(MergedEntryRef, ffi.cast(voidptr_t, rhs))
                );
            end;
        elseif sort_option == "bushou" or sort_option == "radical" then
            cmp_symbols = function (lhs, rhs)
                return compare_by_bushou(
                    ffi.new(MergedEntryRef, ffi.cast(voidptr_t, lhs)),
                    ffi.new(MergedEntryRef, ffi.cast(voidptr_t, rhs))
                );
            end;
        else
            error("unknown sort option: " .. tostring(sort_option));
        end
    end

    for g, v in pairs(target_table) do
        if g == "Numbers" then
            local number_cache = {};
            for _, n_raw in ipairs(v) do
                local is_all_ascii = true;
                local n = ffi.new(MergedEntryRef, ffi.cast(voidptr_t, n_raw));
                for c in n:key_n(0):chars() do
                    if c < 0x30 or c > 0x39 then
                        is_all_ascii = false;
                        break;
                    end
                end
                if is_all_ascii then
                    number_cache[n_raw] = tonumber(tostring(n:key_n(0)));
                end
            end

            table.sort(v, function (l, r)
                if number_cache[l] and number_cache[r] then
                    return number_cache[l] < number_cache[r];
                elseif number_cache[l] then return true;
                elseif number_cache[r] then return false;
                end
                return plain_cmp(l, r);
            end);
        elseif g == "Symbols" then
            table.sort(v, cmp_symbols);
        elseif #g == 1 and latin_group[string.byte(g, 1)] then
            table.sort(v, function (lhs, rhs)
                return compare_by_pinyin(
                    ffi.new(MergedEntryRef, ffi.cast(voidptr_t, lhs)),
                    ffi.new(MergedEntryRef, ffi.cast(voidptr_t, rhs))
                );
            end);
        elseif starts_with(g, "BiHua") then
            table.sort(v, function (lhs, rhs)
                return compare_by_bihua(
                    ffi.new(MergedEntryRef, ffi.cast(voidptr_t, lhs)),
                    ffi.new(MergedEntryRef, ffi.cast(voidptr_t, rhs))
                );
            end);
        elseif starts_with(g, "BuShou") then
            table.sort(v, function (lhs, rhs)
                return compare_by_bushou(
                    ffi.new(MergedEntryRef, ffi.cast(voidptr_t, lhs)),
                    ffi.new(MergedEntryRef, ffi.cast(voidptr_t, rhs))
                );
            end);
        else
            table.sort(v, plain_cmp);
        end
    end
end;

_G.__cindex_write_entries = _G.__cindex_write_entries or function (buf, group_table, target_table)
    if IstOutputStyle:is_nil() then
        error("missing output style");
    end

    local output_style = _G["output_style"] or {};
    for k, v in pairs(IstOutputStyle) do
        local o_v = output_style[k];
        if not o_v then
            output_style[k] = v;
        elseif type(o_v) == type(v) then
            -- pass
        else
            output_style[k] = function() return o_v end;
        end
    end

    buf = ffi.cast(voidptr_t, buf);
    local write_to_output = function (s)
        if ffi.istype(StrRef, s) then
            return __api.write_to_output(buf, s.ptr, s.len);
        elseif ffi.istype(Char, s) then
            local s_buf = char_utf8_buf_t();
            if __api.utf8char.encode_utf8(s.inner, s_buf) then
                return __api.write_to_output(buf, s_buf, s:len_utf8());
            else
                error("invalid char: U+" .. string.format("%X", s.inner));
            end
        elseif type(s) == "string" then
            return __api.write_to_output(buf, s, #s);
        elseif type(s) == "cdata" then
            return nil;
        elseif getmetatable(s) and getmetatable(s).__tostring then
            s = tostring(s);
            return __api.write_to_output(buf, s, #s);
        else
            return nil;
        end
    end;
    local check_write = function (...)
        for _, s in ipairs({...}) do
            if s == nil or write_to_output(s) then
            else error("an error has been raised when writing to output");
            end
        end
    end

    local preamble = output_style:preamble();
    local postamble = output_style:postamble();
    local group_skip = output_style:group_skip();
    local heading_prefix = output_style:heading_prefix();
    local heading_suffix = output_style:heading_suffix();
    local headings_flag = output_style:headings_flag();
    local numhead_positive = output_style:numhead_positive();
    local numhead_negative = output_style:numhead_negative();
    local symhead_positive = output_style:symhead_positive();
    local symhead_negative = output_style:symhead_negative();
    local item_0 = output_style:item_0();
    local item_1 = output_style:item_1();
    local item_2 = output_style:item_2();
    local item_01 = output_style:item_01();
    local item_x1 = output_style:item_x1();
    local item_12 = output_style:item_12();
    local item_x2 = output_style:item_x2();
    local delim_0 = output_style:delim_0();
    local delim_1 = output_style:delim_1();
    local delim_2 = output_style:delim_2();
    local delim_n = output_style:delim_n();
    local delim_r = output_style:delim_r();
    local delim_t = output_style:delim_t();
    local encap_prefix = output_style:encap_prefix();
    local encap_infix = output_style:encap_infix();
    local encap_suffix = output_style:encap_suffix();
    local _page_precedence = output_style:page_precedence();
    local suffix_2p = output_style:suffix_2p();
    local suffix_3p = output_style:suffix_3p();
    local suffix_mp = output_style:suffix_mp();
    local stroke_prefix = output_style:stroke_prefix();
    local stroke_suffix = output_style:stroke_suffix();
    local radical_prefix = output_style:radical_prefix();
    local radical_suffix = output_style:radical_suffix();
    local radical_simplified_flag = tonumber(output_style:radical_simplified_flag());
    local radical_simplified_prefix = output_style:radical_simplified_prefix();
    local radical_simplified_delimiter = output_style:radical_simplified_delimiter();
    local radical_simplified_suffix = output_style:radical_simplified_suffix();

    local write_a_page_range = function (span, commands, l, r)
        if not l then return end
        local replace_r = nil;
        if span == 1 then
        elseif span == 2 then
            replace_r = (suffix_2p and #suffix_2p > 0) and suffix_2p;
        elseif span == 3 then
            replace_r = (suffix_3p and #suffix_3p > 0) and suffix_3p;
        else
            replace_r = (suffix_mp and #suffix_mp > 0) and suffix_mp;
        end
        if replace_r then
            if commands then
                check_write(encap_prefix, commands, encap_infix);
            end
            check_write(l);
            check_write(replace_r);
            if commands then
                check_write(encap_suffix);
            end
        else
            if commands then
                check_write(encap_prefix, commands, encap_infix);
            end
            check_write(l);
            if r then
                check_write(delim_r, r);
            end
            if commands then
                check_write(encap_suffix);
            end
        end
    end;
    local write_pages = function (i, entry)
        local pages_len = entry:pages_count();
        if pages_len == 0 then return false end

        if i == 0 then
            check_write(delim_0);
        elseif i == 1 then
            check_write(delim_1);
        elseif i == 2 then
            check_write(delim_2);
        else
            -- Logger.warn("length of levels is more than 3");
        end

        for idx = 0, pages_len - 1 do
            local page_span = entry:page_span_n(idx);
            if page_span == 0 then
                Logger.warn("empty page value");
                goto continue;
            end
            if idx ~= 0 then check_write(delim_n) end
            write_a_page_range(page_span, entry:page_command_n(idx), entry:pages_raw_n(idx));
            ::continue::
        end

        check_write(delim_t);
        return true;
    end;

    local write_group = function (entries)
        local len_of_entries = #entries;
        local prev_entry;
        local prev_levels_len;
        local missing = 0;
        local max_same_key = 0;
        local levels_len;
        local prev_has_page;

        for i, entry in ipairs(entries) do
            entry = ffi.new(MergedEntryRef, ffi.cast(voidptr_t, entry));

            if i == 1 then
                max_same_key = 0;
            else
                max_same_key = entry:max_same_key(prev_entry);
            end
            levels_len = entry:levels_count();
            missing = levels_len - max_same_key - 1;

            if levels_len == 0 then
                Logger.warn("empty level in group");
            elseif levels_len == 1 then
                check_write(item_0);
                check_write(entry:level_n(0));
                prev_has_page = write_pages(0, entry);
            elseif levels_len == 2 then
                if missing == 1 then
                    check_write(item_0);
                    check_write(entry:level_n(0));
                    prev_levels_len = 1;
                    prev_has_page = false;
                end
                if prev_levels_len == levels_len then
                    check_write(item_1);
                else
                    check_write(prev_has_page and item_01 or item_x1);
                end
                check_write(entry:level_n(1));
                prev_has_page = write_pages(1, entry);
            elseif levels_len == 3 then
                if missing == 2 then
                    check_write(item_0);
                    check_write(entry:level_n(0));
                    prev_levels_len = 1;
                    prev_has_page = false;
                end
                if missing > 0 then
                    if prev_levels_len == 2 then
                        check_write(item_1);
                    else
                        check_write(prev_has_page and item_01 or item_x1);
                    end
                    check_write(entry:level_n(1));
                    prev_levels_len = 2;
                    prev_has_page = false;
                end
                if prev_levels_len == levels_len then
                    check_write(item_2);
                else
                    check_write(prev_has_page and item_12 or item_x2);
                end
                check_write(entry:level_n(2));
                prev_has_page = write_pages(2, entry);
            else
                Logger.warn("length of levels is more than 3: " .. levels_len .. ", last level: " .. tostring(entry:level_n(levels_len - 1)));
            end

            prev_levels_len = levels_len;
            prev_entry = entry;
        end
    end

    check_write(preamble);
    for i, group_name in ipairs(group_table) do
        if i ~= 1 then
            check_write(group_skip);
        end
        if not (headings_flag == 0) then
            check_write(heading_prefix);

            if group_name == "Symbols" then
                if headings_flag < 0 then
                    check_write(symhead_negative);
                elseif headings_flag > 0 then
                    check_write(symhead_positive);
                end
            elseif group_name == "Numbers" then
                if headings_flag < 0 then
                    check_write(numhead_negative);
                elseif headings_flag > 0 then
                    check_write(numhead_positive);
                end
            elseif starts_with(group_name, "BiHua") then
                check_write(stroke_prefix);
                check_write(tostring(sub(group_name, 6)))
                check_write(stroke_suffix);
            elseif starts_with(group_name, "BuShou") then
                check_write(radical_prefix);
                local curr_chars = bushou_to_chars[group_name];
                check_write(Char(curr_chars[1]));
                local simp_len = #curr_chars - 1;
                if radical_simplified_flag > 0 and simp_len > 0 then
                    check_write(radical_simplified_prefix);
                    for s_i = 1, simp_len do
                        if s_i ~= 1 then
                            check_write(radical_simplified_delimiter);
                        end
                        check_write(Char(curr_chars[1 + s_i]));
                    end
                    check_write(radical_simplified_suffix);
                end
                check_write(radical_suffix);
            else
                check_write(group_name);
            end

            check_write(heading_suffix);
        end
        write_group(target_table[group_name]);
    end
    check_write(postamble);
end;
