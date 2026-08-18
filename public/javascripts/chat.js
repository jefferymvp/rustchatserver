$(document).ready(function (e) {
  const $chatBox = $("#input_content");
  const $userList = $("#userList");

  $chatBox.on("input", function () {
    const inputVal = $chatBox.text();
    var users = window.users;

    for (var i = 0; i < users.length; i++) {
      if (users[i] == undefined) {
        users[i] = "null"
      }
    }
    //console.log(users);

    const atIndex = inputVal.lastIndexOf("@");
    if (atIndex !== -1) {
      const query = inputVal.substring(atIndex + 1);
      const filteredUsers = users.filter((user) =>
        user.toLowerCase().includes(query.toLowerCase())
      );
      $userList.empty();
      filteredUsers.forEach((user) => {
        $userList.append(`<li>${user}</li>`);
      });
      const chatBoxOffset = $chatBox.offset();
      $userList
        .css({
          top: chatBoxOffset.top - 300 + $chatBox.outerHeight(),
          left: chatBoxOffset.left + 30,
        })
        .show();
    } else {
      $userList.hide();
    }
  });

  $userList.on("click", "li", function () {
    const selectedUser = $(this).text();
    const inputVal = $chatBox.text();
    const atIndex = inputVal.lastIndexOf("@");
    const newVal = inputVal.substring(0, atIndex + 1) + selectedUser + "　";
    console.log(newVal);
    $chatBox.text(newVal);
    $userList.hide();
    placeCaretAtEnd($chatBox[0]);
  });

  function placeCaretAtEnd(el) {
    el.focus();
    if (
      typeof window.getSelection != "undefined" &&
      typeof document.createRange != "undefined"
    ) {
      const range = document.createRange();
      range.selectNodeContents(el);
      range.collapse(false);
      const sel = window.getSelection();
      sel.removeAllRanges();
      sel.addRange(range);
    }
  }

  $(document).on("click", function (e) {
    if (!$(e.target).closest("#chatBox, #userList").length) {
      $userList.hide();
    }
  });

  $(window).keydown(function (e) {
    if (e.keyCode == 116) {
      if (!confirm("刷新会将所有数据情况，确定要刷新么？")) {
        e.preventDefault();
      }
    }
  });
  var from = $.cookie("user");
  var to = "all";
  $("#from").html(from);
  $("#to").html("所有人");
  window.reconnectd = false;
  $("#input_content").html("");
  if (/Firefox\/\s/.test(navigator.userAgent)) {
    var socket = io.connect({ transports: ["xhr-polling"] });
  } else if (/MSIE (\d+.\d+);/.test(navigator.userAgent)) {
    var socket = io.connect({ transports: ["jsonp-polling"] });
  } else {
    var socket = io.connect();
  }
  socket.emit("online", JSON.stringify({ user: from }));
  socket.on("disconnect", function () {
    var msg = '<div style="color:#f00">SYSTEM:连接服务器失败</div>';
    addMsg(msg);
    $("#list").empty();
  });
  socket.on("reconnect", function () {
    socket.emit("online", JSON.stringify({ user: from }));
    var msg = '<div style="color:#f00">SYSTEM:重新连接服务器</div>';
    window.reconnectd = true;
    addMsg(msg);
    //经过5秒后再将reconnectd设置成false
    setTimeout(function () {
      window.reconnectd = false;
    }, 8000);
  });
  socket.on("notice", function (data) {
    var data = JSON.parse(data);
    var time = getTimeShow(data.time);
    var msg = data.msg;
    // 添加 white-space: pre-wrap; 样式
    var msg = '<div style="color:#f00; white-space: pre-wrap;">SYSTEM(' + time + "):<br/>" + msg + "</div>";
    addMsg(msg);
    play_ring("/ring/online.wav");
  });
  socket.on("system", function (data) {
    var data = JSON.parse(data);
    var time = getTimeShow(data.time);
    var msg = "";
    if (!window.reconnectd) {
      //重连成功后的通知不显示
      if (data.type == "online") {
        msg += "用户 " + data.msg + " 上线了！";
      } else if (data.type == "offline") {
        msg += "用户 " + data.msg + " 下线了！";
      } else if (data.type == "in") {
        msg += "你进入了聊天室！";
      } else {
        msg += "未知系统消息！";
      }
      var msg =
        '<div style="color:#f00">SYSTEM(' + time + "):" + msg + "</div>";
      addMsg(msg);
      play_ring("/ring/online.wav");
    }
  });
  socket.on("userflush", function (data) {
    var data = JSON.parse(data);
    var users = data.users;
    window.users = users;
    flushUsers(users);
  });

  socket.on("clear_history_client", function () {
    $("#contents").empty();
    console.log("服务器下发了清空指令，已清空本地面板");
  });

  // 文件上传结果通知：仅弹出系统通知，不写入聊天记录
  socket.on("upload_ack", function (data) {
    var payload = typeof data === "string" ? JSON.parse(data) : data;
    var msg = payload.msg || "文件操作完成";
    showNotice(msg);
  });

  socket.on("image", (data) => {
    console.log(data);
    var data = JSON.parse(data);
    var time = getTimeShow(new Date());
    var imageData = data.msg;
    var src = `data:image/png;base64,${imageData}`;
    var msg =
      "<div>" +
      data.from +
      "(" +
      time +
      ")上图：<br/><img onclick='popupImg(this.src);return false;' src='" +
      src +
      "' style='max-width: 300px; max-height: 300px;'></img></div>";
    addMsg(msg);
  });

  socket.on("say", function (msgData) {
    if (typeof msgData === "string") {
      try {
        msgData = JSON.parse(msgData);
      } catch (e) {
        console.error("解析 say 消息异常:", e);
      }
    }
    if (!msgData || !msgData.data) {
      console.warn("收到空或不合法的 say 消息:", msgData);
      return;
    }

    var time = getTimeShow(msgData.time || new Date());
    var data = msgData.data;
    var sender = data.from || "匿名用户";
    var target = data.to || "all";
    var content = data.msg || "";

    if (target == "all") {
      addMsg(
        "<div>" + sender + "(" + time + ")说：<br/>" + content + "</div>"
      );
      //防止重连后疯狂通知
      if (sender != from && !window.reconnectd && msgData.notice) {
        showNotice(sender + "：" + content);
        play_ring("/ring/msg.wav");
      }
    } else if (sender == from) {
      addMsg(
        "<div>我(" + time + ")对" + target + "说：<br/>" + content + "</div>"
      );
    } else if (target == from) {
      addMsg(
        "<div>" +
        sender +
        "(" +
        time +
        ")对我说：<br/>" +
        content +
        "</div>"
      );
      if (!window.reconnectd && msgData.notice) {
        showNotice(sender + "说：" + content);
        play_ring("/ring/msg.wav");
      }
    }
    $("img").click(function () {
      popupImg(this.src);
      return false;
    });
  });

  // 网页内悬浮 Toast 提示（不受浏览器通知权限限制）
  function showToast(msg) {
    var $toast = $("#app_toast");
    if ($toast.length === 0) {
      $toast = $(
        '<div id="app_toast" style="position: fixed; top: 25px; left: 50%; transform: translateX(-50%); background: #1a1a1a; color: #fff; padding: 10px 24px; border-radius: 20px; font-size: 14px; font-weight: 500; z-index: 99999; box-shadow: 0 4px 16px rgba(0,0,0,0.25); display: none; transition: all 0.3s ease; pointer-events: none; border: 1px solid rgba(255,255,255,0.15);"></div>'
      );
      $("body").append($toast);
    }
    $toast.text(msg).stop(true, true).fadeIn(200).delay(2800).fadeOut(400);
  }

  function showNotice(msg) {
    // 1. 始终触发页面内的浮窗提示
    showToast(msg);
    play_ring("/ring/online.wav");

    // 2. 尝试触发浏览器系统桌面通知
    if (typeof Notification !== "undefined") {
      var newNotify = function () {
        var notification = new Notification("系统通知", {
          dir: "auto",
          requireInteraction: false,
          body: msg,
        });
        notification.onclick = function () {
          window.focus();
        };
      };
      if (Notification.permission === "granted") {
        newNotify();
      } else if (Notification.permission !== "denied") {
        Notification.requestPermission(function (perm) {
          if (perm === "granted") {
            newNotify();
          }
        });
      }
    }
  }

  function addMsg(msg) {
    var $msg = $("<div>" + msg + "</div>");
    $msg.find("img").click(function () {
      if (typeof popupImg === "function") {
        popupImg(this.src);
      }
      return false;
    });
    $("#contents").append($msg);
    $("#contents").scrollTop($("#contents")[0].scrollHeight);
  }
  function flushUsers(users) {
    var ulEle = $("#list");
    ulEle.empty();
    ulEle.append(
      '<li title="双击聊天" alt="all" onselectstart="return false">所有人</li>'
    );
    for (var i = 0; i < users.length; i++) {
      ulEle.append(
        '<li alt="' +
        users[i] +
        '" title="双击聊天" onselectstart="return false">' +
        users[i] +
        "</li>"
      );
    }
    //双击对某人聊天
    $("#list > li").dblclick(function (e) {
      if ($(this).attr("alt") != from) {
        to = $(this).attr("alt");
        show_say_to();
      }
    });
    show_say_to();
  }
  $("#input_content").keydown(function (e) {
    if (e.shiftKey && e.which == 13) {
      $("#input_content").append("<br/>");
    } else if (e.which == 13) {
      e.preventDefault();
      say();
    }
  });
  $("#say").click(function (e) {
    say();
  });
  $("#clear_btn").click(function (e) {
    if (confirm("确定要清空所有聊天和播报记录吗？")) {
      socket.emit("clear_history");
    }
    $(this).blur(); // 强制失去焦点，恢复样式
  });
  $("#image-input").change(function (e) {
    const file = e.target.files[0];

    if (file) {
      const reader = new FileReader();
      reader.onload = (event) => {
        console.log("[文件上传] 文件名:", file.name, " 大小:", file.size);
        const dataUrl = event.target.result;
        // 将文件数据以 Base64 格式通过 WebSocket 传给服务端保存
        socket.emit('fileUpload', { fileName: file.name, fileBuffer: dataUrl });
        socket.emit(
          "say",
          JSON.stringify({
            to: to || "all",
            from: from || "匿名用户",
            msg: "发送文件:<a target='_blank' href='/doc/" + encodeURIComponent(file.name) + "'>" + file.name + "</a>",
          })
        );
      };
      reader.readAsDataURL(file);
      // 重置 input value 以便支持重复上传同名或同一文件
      $(this).val("");
    }
  });
  function say() {
    var $input = $("#input_content");
    var html = $input.html();
    var hasImg = $input.find("img").length > 0 || /<img\s+/i.test(html);
    var text = $input.text().trim();
    
    // 如果既没有文字，也没有包含图片标签，则不发送
    if (!html || (!hasImg && text === "")) {
      return;
    }
    if (!from) {
      from = $.cookie("user") || "匿名用户";
    }
    var target = to || "all";
    var payload = JSON.stringify({ to: target, from: from, msg: html });
    console.log("[say] 发送 payload (包含图片或文字):", { target: target, from: from, hasImg: hasImg });
    socket.emit("say", payload);
    $input.html("");
    $input.focus();
  }
  function sendimg() {
    socket.emit(
      "image",
      JSON.stringify({ to: to, from: from, msg: $("#input_content").html() })
    );
    $("#input_content").html("");
    $("#input_content").focus();
  }
  //显示正在对谁说话
  function show_say_to() {
    $("#from").html(from);
    $("#to").html(to == "all" ? "所有人" : to);
    var users = $("#list > li");
    for (var i = 0; i < users.length; i++) {
      if ($(users[i]).attr("alt") == to) {
        $(users[i]).addClass("sayingto");
      } else {
        $(users[i]).removeClass("sayingto");
      }
    }
  }
  function play_ring(url) {
    var embed =
      '<embed id="ring" src="' +
      url +
      '" loop="0" autostart="true" hidden="true" style="height:0px; width:0px;0px;"></embed>';
    $("#ring").html(embed);
  }
  function getTimeShow(time) {
    var dt = new Date(time);
    time =
      dt.getFullYear() +
      "-" +
      (dt.getMonth() + 1) +
      "-" +
      dt.getDate() +
      " " +
      dt.getHours() +
      ":" +
      (dt.getMinutes() < 10 ? "0" + dt.getMinutes() : dt.getMinutes()) +
      ":" +
      (dt.getSeconds() < 10 ? "0" + dt.getSeconds() : dt.getSeconds());
    return time;
  }
  $.cookie("isLogin", true);

  // H5 切换显示用户列表
  $("#user_toggle_btn").click(function () {
    $("#users_online").fadeToggle(200);
  });
});

